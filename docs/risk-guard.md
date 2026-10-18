# RiskGuard — Circuit Breaker for Trading Systems

RiskGuard is a per-token circuit breaker that prevents catastrophic losses
by detecting dangerous trading patterns in real time.

## What it does

Every time your trading engine places an order, you call `RiskGuard::check_fast()`.
RiskGuard tracks the recent history of large trades for each token and trips
a circuit breaker when patterns become dangerous.

## Core concept

A **circuit breaker** (`TrippedUntil`) temporarily blocks trading in a token
after a sequence of large trades triggers it. Once the trip duration expires,
trading resumes automatically.

## Checks

### `check_fast` — Sequence Detection

Detects when many large trades occur in rapid succession:

```
sequence_window: 40s
consecutive_trigger: 5 large trades in 40s → trip for 5 hours
```

"Large" means `shares > large_trade_shares` (default: 2000 shares).

### `check_with_book` — Depth Check

When the order book is thin (not enough depth beyond the fill price),
a large trade can move the price significantly against you.

```
min_depth_beyond_usd: $200
If depth 200bps beyond whale price < $200 → BLOCK
```

## State

```rust
pub struct RiskGuard {
    tokens: FxHashMap<String, TokenState>,  // per-token state
    config: RiskGuardConfig,
}

struct TokenState {
    large_trades: Vec<(Instant, f64)>,     // timestamp + size
    tripped_until: Option<Instant>,          // circuit breaker
}
```

State is **in-memory only** — after a restart, the circuit breaker resets.
For production, persist `tripped_until` to disk.

## Usage

```rust
use pm_core::{RiskGuard, RiskGuardConfig, Decision};

let config = RiskGuardConfig::default();
let mut guard = RiskGuard::new(config);

// Before placing a trade
match guard.check_fast("POLITICS-2024-1", 5000.0) {
    Decision::Allow => { /* place order */ }
    Decision::Block(reason) => { /* log + skip */ }
}

// With book depth data
match guard.check_with_book("POLITICS-2024-1", 5, 100.0) {
    Decision::Allow => { /* place order */ }
    Decision::Block(reason) => { /* log + skip */ }
}
```

## Configuration

```rust
RiskGuardConfig {
    large_trade_shares: 2000.0,      // what counts as "large"
    consecutive_trigger: 5,           // large trades needed to trip
    sequence_window: 40s,             // time window for sequence
    min_depth_beyond_usd: 200.0,      // min book depth for check_with_book
    trip_duration: 5 hours,           // how long to stay tripped
}
```

## Integration with trading engines

In a copy-trading bot, the call sequence is:

```
whale_signal → check RiskGuard → check MarketValidator → place order
```

RiskGuard is the **first line of defense** — it runs before market validation
and costs almost nothing (O(1) with a small vector).

## Design notes

- Uses `FxHashMap` + `Vec` (no mutex, no async) — designed to be called
  from a synchronous hot path
- Trades are stored as `(Instant, shares)` pairs, old entries pruned on each call
- Tripped state is monotonic — once tripped, stays tripped until `tripped_until`
