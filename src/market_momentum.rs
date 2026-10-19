// =============================================================================
// MARKET MOMENTUM SCORE — Market Conditions Scoring
// =============================================================================
//
// Position in pipeline:
// [MARKET VALIDATOR] → PASS → [MARKET MOMENTUM SCORE] → [ENTRY SCORER]
//
// Different from Validator, this scores — it's one of the Entry Scorer signals.
// If score is too low, can alone downgrade action from COPY to REDUCE or SKIP.
//
// Purpose:
// Measure if the market itself is in favorable conditions for entry,
// regardless of whale behavior. A good whale in a bad market is still a bad entry.
//
// Author: Generated from spec
// Date: 2026-03-05
// =============================================================================

use serde::{Deserialize, Serialize};

/// Input context for momentum calculation
#[derive(Debug, Clone, Copy)]
pub struct MomentumContext {
    pub volume_24h: f64,
    pub volume_7d_avg: f64,
    pub spread_pct_now: f64,
    pub spread_pct_1h_ago: f64,
    pub price_1h_ago: f64,
    pub price_now: f64,
    pub whale_fill_price: f64,
    pub num_unique_traders_24h: u32,
}

/// Sub-scores breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumSubScores {
    pub volume_acceleration: f64,
    pub spread_trend: f64,
    pub price_stability: f64,
}

/// Result of momentum calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumResult {
    pub momentum_score: f64,
    pub force_skip: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_skip_reason: Option<String>,
    pub sub_scores: MomentumSubScores,
    pub raw_metrics: MomentumRawMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumRawMetrics {
    pub volume_ratio: f64,
    pub spread_delta: f64,
    pub price_move_pct: f64,
}

/// Weights for sub-score combination
pub struct MomentumWeights {
    pub volume_weight: f64,
    pub spread_weight: f64,
    pub stability_weight: f64,
}

impl Default for MomentumWeights {
    fn default() -> Self {
        Self {
            volume_weight: 0.40,
            spread_weight: 0.30,
            stability_weight: 0.30,
        }
    }
}

/// Score volume acceleration (0.0 to 1.0)
fn score_volume_acceleration(ratio: f64) -> f64 {
    if ratio > 2.0 {
        1.0 // volume > 2x average: hot market
    } else if ratio >= 1.5 {
        0.85
    } else if ratio >= 1.0 {
        0.65 // normal
    } else if ratio >= 0.5 {
        0.35 // below average: caution
    } else {
        0.10 // market dying
    }
}

/// Score spread trend (0.0 to 1.0)
// delta > 0 means spread tightened (good), delta < 0 means spread widened (bad)
fn score_spread_trend(delta: f64) -> f64 {
    if delta > 2.0 {
        1.0 // spread tightened a lot: excellent
    } else if delta >= 0.5 {
        0.80
    } else if delta >= -0.5 {
        0.60 // stable
    } else if delta >= -2.0 {
        0.30 // widening: care
    } else {
        0.05 // widening fast: likely adverse event
    }
}

/// Score price stability (0.0 to 1.0)
// Returns (score, is_hard_block)
fn score_price_stability(price_move_pct: f64) -> (f64, bool) {
    if price_move_pct < 2.0 {
        (1.0, false) // calm market, whale is catalyst
    } else if price_move_pct < 5.0 {
        (0.70, false)
    } else if price_move_pct < 10.0 {
        (0.35, false) // already moved
    } else if price_move_pct < 20.0 {
        (0.10, false)
    } else {
        (0.0, true) // HARD BLOCK: move already happened
    }
}

impl MomentumResult {
    /// Compute momentum score from context
    pub fn compute(ctx: &MomentumContext, weights: &MomentumWeights) -> Self {
        // Create span for momentum calculation
        let _span = tracing::debug_span!(
            "momentum_calculation",
            event_type = "momentum_calculated",
            volume_24h = ctx.volume_24h,
            volume_7d_avg = ctx.volume_7d_avg,
            spread_pct_now = ctx.spread_pct_now,
            spread_pct_1h_ago = ctx.spread_pct_1h_ago,
            price_move_pct = 0.0, // will be updated below
        );
        let _enter = _span.enter();

        // Calculate raw metrics
        let volume_ratio = if ctx.volume_7d_avg > 0.0 {
            ctx.volume_24h / ctx.volume_7d_avg
        } else {
            1.0 // default to normal if no history
        };

        let spread_delta = ctx.spread_pct_1h_ago - ctx.spread_pct_now;
        // positive = spread tightened (good)

        let price_move_pct = if ctx.price_1h_ago > 0.0 {
            ((ctx.price_now - ctx.price_1h_ago) / ctx.price_1h_ago * 100.0).abs()
        } else {
            0.0
        };

        // Update span with actual price move
        tracing::debug!(price_move_pct = price_move_pct);

        // RULE: Hard block if price moved > 20%
        let (score_stability, force_skip) = score_price_stability(price_move_pct);

        if force_skip {
            tracing::warn!(
                event_type = "momentum_skip",
                reason = "PRICE_ALREADY_MOVED",
                price_move_pct = price_move_pct,
                decision_path = "momentum_calculation"
            );
            return Self {
                momentum_score: 0.0,
                force_skip: true,
                force_skip_reason: Some("PRICE_ALREADY_MOVED".to_string()),
                sub_scores: MomentumSubScores {
                    volume_acceleration: 0.0,
                    spread_trend: 0.0,
                    price_stability: 0.0,
                },
                raw_metrics: MomentumRawMetrics {
                    volume_ratio,
                    spread_delta,
                    price_move_pct,
                },
            };
        }

        // Calculate sub-scores
        let score_volume = score_volume_acceleration(volume_ratio);
        let score_spread = score_spread_trend(spread_delta);

        // Weighted final score
        let momentum_score = score_volume * weights.volume_weight
            + score_spread * weights.spread_weight
            + score_stability * weights.stability_weight;

        // Log successful momentum calculation
        tracing::info!(
            event_type = "momentum_calculated",
            momentum_score = momentum_score,
            volume_ratio = volume_ratio,
            spread_delta = spread_delta,
            price_move_pct = price_move_pct,
            volume_acceleration = score_volume,
            spread_trend = score_spread,
            price_stability = score_stability,
            decision_path = "momentum_calculation_complete"
        );

        Self {
            momentum_score,
            force_skip: false,
            force_skip_reason: None,
            sub_scores: MomentumSubScores {
                volume_acceleration: score_volume,
                spread_trend: score_spread,
                price_stability: score_stability,
            },
            raw_metrics: MomentumRawMetrics {
                volume_ratio,
                spread_delta,
                price_move_pct,
            },
        }
    }
}

/// How momentum score integrates into entry scorer
/// Entry Scorer weights with momentum (total = 1.00):
/// - whale_track_record: 0.20
/// - price_slippage: 0.20
/// - spread_health: 0.15
/// - momentum_score: 0.20 (NEW - replaces old volume_momentum)
/// - time_value: 0.10
/// - position_crowding: 0.10
/// - price_zone: 0.05
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_acceleration() {
        assert_eq!(score_volume_acceleration(3.0), 1.0);
        assert_eq!(score_volume_acceleration(1.8), 0.85);
        assert_eq!(score_volume_acceleration(1.2), 0.65);
        assert_eq!(score_volume_acceleration(0.7), 0.35);
        assert_eq!(score_volume_acceleration(0.3), 0.10);
    }

    #[test]
    fn test_spread_trend() {
        assert_eq!(score_spread_trend(3.0), 1.0);
        assert_eq!(score_spread_trend(1.0), 0.80);
        assert_eq!(score_spread_trend(0.0), 0.60);
        assert_eq!(score_spread_trend(-1.0), 0.30);
        assert_eq!(score_spread_trend(-3.0), 0.05);
    }

    #[test]
    fn test_price_stability() {
        let (score, block) = score_price_stability(1.0);
        assert_eq!(score, 1.0);
        assert!(!block);

        let (score, block) = score_price_stability(25.0);
        assert_eq!(score, 0.0);
        assert!(block); // hard block!
    }

    #[test]
    fn test_good_momentum() {
        let ctx = MomentumContext {
            volume_24h: 16000.0,
            volume_7d_avg: 10000.0, // 1.6x
            spread_pct_now: 5.0,
            spread_pct_1h_ago: 6.2, // +1.2% tightening
            price_1h_ago: 0.50,
            price_now: 0.5175, // 3.5% move
            whale_fill_price: 0.50,
            num_unique_traders_24h: 150,
        };

        let weights = MomentumWeights::default();
        let result = MomentumResult::compute(&ctx, &weights);

        assert!(!result.force_skip);
        assert!(result.momentum_score > 0.7);
    }

    #[test]
    fn test_price_already_moved() {
        let ctx = MomentumContext {
            volume_24h: 10000.0,
            volume_7d_avg: 10000.0,
            spread_pct_now: 5.0,
            spread_pct_1h_ago: 5.0,
            price_1h_ago: 0.50,
            price_now: 0.65, // 30% move!
            whale_fill_price: 0.50,
            num_unique_traders_24h: 150,
        };

        let weights = MomentumWeights::default();
        let result = MomentumResult::compute(&ctx, &weights);

        assert!(result.force_skip);
        assert_eq!(
            result.force_skip_reason,
            Some("PRICE_ALREADY_MOVED".to_string())
        );
        assert_eq!(result.momentum_score, 0.0);
    }
}
