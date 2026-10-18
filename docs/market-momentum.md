# MarketMomentum — 3-Factor Market Sentiment Score

MarketMomentum computes a composite score (0.0–1.0) describing how "hot" a market is.
Use it to prioritize markets, size positions, or decide entry timing.

## Three Sub-scores

### Volume Acceleration (40%)

```
ratio = volume_24h / volume_7d_avg
score = tanh(ratio - 1)  → 0.0 (cold) to 1.0 (surge)
```

Volume significantly above the 7-day average indicates increased interest —
often correlated with upcoming events or news catalysts.

### Spread Trend (35%)

```
delta = spread_pct_now - spread_pct_1h_ago
score = tanh(-delta)  → 1.0 (tightening) to 0.0 (widening)
```

Spreads tightening = market is becoming more liquid (positive signal).
Spreads widening = market is becoming toxic (negative signal).

### Price Stability (25%)

```
move = |price_now - price_1h_ago| / price_1h_ago
score = tanh(-move * 10)  → 1.0 (stable) to 0.0 (volatile)
```

Stable prices = predictable market, lower slippage risk.
Volatile prices = uncertain outcome, higher execution risk.

## Composite Score

```
momentum = 0.40 * volume_accel + 0.35 * spread_trend + 0.25 * price_stability
```

Range: 0.0 (very cold) to 1.0 (very hot).

## Usage

```rust
use pm_core::{MarketMomentum, MomentumContext, MomentumWeights};

let weights = MomentumWeights::default();
let ctx = MomentumContext {
    volume_24h:         100_000.0,
    volume_7d_avg:       50_000.0,
    spread_pct_now:       0.005,
    spread_pct_1h_ago:    0.020,
    price_1h_ago:        0.55,
    price_now:           0.57,
    whale_fill_price:    0.57,
    num_unique_traders_24h: 15,
};

let result = MomentumResult::compute(&ctx, &weights);
println!("Momentum: {:.2}", result.momentum_score);
```

## When to use it

- **Entry sizing**: Higher momentum → larger position (with risk guard)
- **Market prioritization**: Sort markets by momentum, trade the hottest first
- **Skip conditions**: Momentum < 0.3 → skip market entirely
- **Exit timing**: Momentum dropping fast → consider closing position

## Demo

```bash
cargo run --bin demo
```

The demo computes momentum for three market regimes: calm, surging, and weakening.
