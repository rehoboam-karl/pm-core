// =============================================================================
// MARKET VALIDATOR — Binary Gate Before Entry Scorer
// =============================================================================
//
// Position in pipeline:
// Whale trade detected → [MARKET VALIDATOR] → PASS/FAIL → [ENTRY SCORER] → [SESSION GUARD]
//
// This is a binary gate: PASS or FAIL. No middle ground.
// If FAIL, silently discard — doesn't count as consecutive block for whale
// (except SLIPPAGE_TOO_HIGH).
//
// Author: Generated from spec
// Date: 2026-03-05
// =============================================================================

use serde::{Deserialize, Serialize};

/// Hard rules that must all pass for the trade to proceed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidatorFailReason {
    MarketClosed,
    SpreadTooWide,
    InsufficientLiquidity,
    MarketExpiring,
    SlippageTooHigh,
}

impl ValidatorFailReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MarketClosed => "MARKET_CLOSED",
            Self::SpreadTooWide => "SPREAD_TOO_WIDE",
            Self::InsufficientLiquidity => "INSUFFICIENT_LIQUIDITY",
            Self::MarketExpiring => "MARKET_EXPIRING",
            Self::SlippageTooHigh => "SLIPPAGE_TOO_HIGH",
        }
    }
}

/// Input metrics for validation
#[derive(Debug, Clone, Copy)]
pub struct ValidatorMetrics {
    pub live: bool,
    pub spread_pct: f64,
    pub book_depth_usd: f64,
    pub hours_remaining: f64,
    pub whale_fill_price: f64,
    pub current_best_ask: f64,
}

/// Configuration for validator thresholds
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    pub max_spread_pct: f64,
    pub min_book_depth_usd: f64,
    pub min_hours_remaining: f64,
    pub max_slippage_pct: f64,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            max_spread_pct: 10.0,       // 10% max spread
            min_book_depth_usd: 2000.0, // $2000 min depth
            min_hours_remaining: 6.0,   // 6 hours min
            max_slippage_pct: 15.0,     // 15% max slippage
        }
    }
}

/// Result of market validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorResult {
    pub pass: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_reason: Option<String>,
    pub counts_as_whale_block: bool,
    pub metrics: ValidatorMetricsOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorMetricsOutput {
    pub live: bool,
    pub spread_pct: f64,
    pub book_depth_usd: f64,
    pub hours_remaining: f64,
    pub slippage_pct: f64,
}

impl ValidatorResult {
    /// Create a PASS result
    pub fn pass(metrics: &ValidatorMetrics) -> Self {
        Self {
            pass: true,
            fail_reason: None,
            counts_as_whale_block: false,
            metrics: ValidatorMetricsOutput {
                live: metrics.live,
                spread_pct: metrics.spread_pct,
                book_depth_usd: metrics.book_depth_usd,
                hours_remaining: metrics.hours_remaining,
                slippage_pct: calculate_slippage(metrics),
            },
        }
    }

    /// Create a FAIL result
    pub fn fail(reason: ValidatorFailReason, metrics: &ValidatorMetrics) -> Self {
        // Only SLIPPAGE_TOO_HIGH counts against the whale
        let counts = matches!(reason, ValidatorFailReason::SlippageTooHigh);

        Self {
            pass: false,
            fail_reason: Some(reason.as_str().to_string()),
            counts_as_whale_block: counts,
            metrics: ValidatorMetricsOutput {
                live: metrics.live,
                spread_pct: metrics.spread_pct,
                book_depth_usd: metrics.book_depth_usd,
                hours_remaining: metrics.hours_remaining,
                slippage_pct: calculate_slippage(metrics),
            },
        }
    }
}

/// Calculate slippage percentage
fn calculate_slippage(metrics: &ValidatorMetrics) -> f64 {
    if metrics.whale_fill_price <= 0.0 || metrics.current_best_ask <= 0.0 {
        return 0.0;
    }
    ((metrics.current_best_ask - metrics.whale_fill_price) / metrics.whale_fill_price) * 100.0
}

/// Run all validation rules in order
pub fn validate(config: &ValidatorConfig, metrics: &ValidatorMetrics) -> ValidatorResult {
    // Create span for market validation
    let _span = tracing::debug_span!(
        "market_validation",
        event_type = "market_validated",
        live = metrics.live,
        spread_pct = metrics.spread_pct,
        book_depth_usd = metrics.book_depth_usd,
        hours_remaining = metrics.hours_remaining,
    );
    let _enter = _span.enter();

    // RULE 1: LIVE CHECK
    if !metrics.live {
        tracing::warn!(
            event_type = "market_rejected",
            reason = "MARKET_CLOSED",
            spread_pct = metrics.spread_pct,
            book_depth_usd = metrics.book_depth_usd,
            decision_path = "market_validation"
        );
        return ValidatorResult::fail(ValidatorFailReason::MarketClosed, metrics);
    }

    // RULE 2: SPREAD CHECK
    if metrics.spread_pct > config.max_spread_pct {
        tracing::warn!(
            event_type = "market_rejected",
            reason = "SPREAD_TOO_WIDE",
            spread_pct = metrics.spread_pct,
            max_spread_pct = config.max_spread_pct,
            decision_path = "market_validation"
        );
        return ValidatorResult::fail(ValidatorFailReason::SpreadTooWide, metrics);
    }

    // RULE 3: LIQUIDITY CHECK
    if metrics.book_depth_usd < config.min_book_depth_usd {
        tracing::warn!(
            event_type = "market_rejected",
            reason = "INSUFFICIENT_LIQUIDITY",
            book_depth_usd = metrics.book_depth_usd,
            min_book_depth_usd = config.min_book_depth_usd,
            decision_path = "market_validation"
        );
        return ValidatorResult::fail(ValidatorFailReason::InsufficientLiquidity, metrics);
    }

    // RULE 4: TIME CHECK
    if metrics.hours_remaining < config.min_hours_remaining {
        tracing::warn!(
            event_type = "market_rejected",
            reason = "MARKET_EXPIRING",
            hours_remaining = metrics.hours_remaining,
            min_hours_remaining = config.min_hours_remaining,
            decision_path = "market_validation"
        );
        return ValidatorResult::fail(ValidatorFailReason::MarketExpiring, metrics);
    }

    // RULE 5: SLIPPAGE CHECK
    let slippage_pct = calculate_slippage(metrics);
    if slippage_pct > config.max_slippage_pct {
        tracing::warn!(
            event_type = "market_rejected",
            reason = "SLIPPAGE_TOO_HIGH",
            slippage_pct = slippage_pct,
            max_slippage_pct = config.max_slippage_pct,
            decision_path = "market_validation"
        );
        return ValidatorResult::fail(ValidatorFailReason::SlippageTooHigh, metrics);
    }

    // All rules passed
    tracing::info!(
        event_type = "market_validated",
        spread_pct = metrics.spread_pct,
        book_depth_usd = metrics.book_depth_usd,
        hours_remaining = metrics.hours_remaining,
        slippage_pct = slippage_pct,
        decision_path = "market_validation_passed"
    );

    ValidatorResult::pass(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_closed() {
        let config = ValidatorConfig::default();
        let metrics = ValidatorMetrics {
            live: false,
            spread_pct: 5.0,
            book_depth_usd: 5000.0,
            hours_remaining: 24.0,
            whale_fill_price: 0.50,
            current_best_ask: 0.55,
        };

        let result = validate(&config, &metrics);
        assert!(!result.pass);
        assert_eq!(result.fail_reason, Some("MARKET_CLOSED".to_string()));
        assert!(!result.counts_as_whale_block);
    }

    #[test]
    fn test_spread_too_wide() {
        let config = ValidatorConfig::default();
        let metrics = ValidatorMetrics {
            live: true,
            spread_pct: 15.0, // > 10%
            book_depth_usd: 5000.0,
            hours_remaining: 24.0,
            whale_fill_price: 0.50,
            current_best_ask: 0.55,
        };

        let result = validate(&config, &metrics);
        assert!(!result.pass);
        assert_eq!(result.fail_reason, Some("SPREAD_TOO_WIDE".to_string()));
        assert!(!result.counts_as_whale_block);
    }

    #[test]
    fn test_insufficient_liquidity() {
        let config = ValidatorConfig::default();
        let metrics = ValidatorMetrics {
            live: true,
            spread_pct: 5.0,
            book_depth_usd: 1000.0, // < $2000
            hours_remaining: 24.0,
            whale_fill_price: 0.50,
            current_best_ask: 0.55,
        };

        let result = validate(&config, &metrics);
        assert!(!result.pass);
        assert_eq!(
            result.fail_reason,
            Some("INSUFFICIENT_LIQUIDITY".to_string())
        );
        assert!(!result.counts_as_whale_block);
    }

    #[test]
    fn test_market_expiring() {
        let config = ValidatorConfig::default();
        let metrics = ValidatorMetrics {
            live: true,
            spread_pct: 5.0,
            book_depth_usd: 5000.0,
            hours_remaining: 3.0, // < 6h
            whale_fill_price: 0.50,
            current_best_ask: 0.55,
        };

        let result = validate(&config, &metrics);
        assert!(!result.pass);
        assert_eq!(result.fail_reason, Some("MARKET_EXPIRING".to_string()));
        assert!(!result.counts_as_whale_block);
    }

    #[test]
    fn test_slippage_too_high() {
        let config = ValidatorConfig::default();
        let metrics = ValidatorMetrics {
            live: true,
            spread_pct: 5.0,
            book_depth_usd: 5000.0,
            hours_remaining: 24.0,
            whale_fill_price: 0.50,
            current_best_ask: 0.60, // 20% slippage
        };

        let result = validate(&config, &metrics);
        assert!(!result.pass);
        assert_eq!(result.fail_reason, Some("SLIPPAGE_TOO_HIGH".to_string()));
        assert!(result.counts_as_whale_block); // This DOES count!
    }

    #[test]
    fn test_all_pass() {
        let config = ValidatorConfig::default();
        let metrics = ValidatorMetrics {
            live: true,
            spread_pct: 8.5,
            book_depth_usd: 4200.0,
            hours_remaining: 72.0,
            whale_fill_price: 0.50,
            current_best_ask: 0.545, // 9% slippage
        };

        let result = validate(&config, &metrics);
        assert!(result.pass);
        assert!(result.fail_reason.is_none());
        assert!(!result.counts_as_whale_block);
    }
}
