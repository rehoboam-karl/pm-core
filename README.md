# pm-core

> ⚠️ **This is not financial advice.** Cryptocurrency and prediction market trading involve substantial risk of loss. This software is for educational and informational purposes only. Use at your own risk.

[![CI](https://github.com/rehoboam-karl/pm-core/actions/workflows/ci.yml/badge.svg)](https://github.com/rehoboam-karl/pm-core/actions)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

`pm-core` is a Rust library of components for building Polymarket trading bots: market quality validation, momentum scoring, circuit breakers, position tracking, and trade journaling. Used on the author's own infrastructure.

**No API keys. No async runtime. Runs anywhere.**

---

## Modules

| Module | What it does |
|--------|-------------|
| `market_validator` | 5-rule binary gate: spread, liquidity, slippage, hours, live |
| `market_momentum` | 3-factor composite momentum score (volume, spread trend, price) |
| `risk_guard` | Per-token circuit breaker with sequence detection |
| `exposure_tracker` | In-memory position tracking with real-time P&L |
| `trade_journal` | Async non-blocking CSV trade logger |
| `market_cache` | Thread-safe price/volume cache with TTL |

---

## Quick Start

```toml
# Cargo.toml
[dependencies]
pm-core = { git = "https://github.com/rehoboam-karl/pm-core" }
```

Or clone and use locally:

```bash
git clone https://github.com/rehoboam-karl/pm-core
cd pm-core
cargo build
cargo test
cargo run --bin demo   # runs the demo with synthetic data
```

## Demo

The `demo` binary shows all modules with synthetic data — no API keys needed:

```bash
cargo run --bin demo
```

```
=== pm-core Demo ===

--- MarketValidator ---
  ✓ Market passes validation
  ✗ Correctly rejected: Some("SPREAD_TOO_WIDE")

--- RiskGuard ---
  Small trade (30 shares): Allow
  Large trade (5000 shares): Allow
  With book depth check: Block

--- TradeJournal ---
  Entries written: (noop mode — see real journal in production)

--- MarketMomentum ---
  Period 1 momentum: 0.7 (calm market)
  Period 2 momentum: 0.7 (volume surge)
  Period 3 momentum: 0.5 (spread widening)
```

---

## Usage Examples

### MarketValidator

```rust
use pm_core::{ValidatorConfig, ValidatorMetrics, validate};

let cfg = ValidatorConfig::default();
let metrics = ValidatorMetrics {
    live: true,
    spread_pct: 2.0,
    book_depth_usd: 10_000.0,
    hours_remaining: 48.0,
    whale_fill_price: 0.55,
    current_best_ask: 0.551,
};

let result = validate(&cfg, &metrics);
assert!(result.pass);
```

### RiskGuard

```rust
use pm_core::{RiskGuard, RiskGuardConfig, Decision};

let config = RiskGuardConfig::default();
let mut guard = RiskGuard::new(config);

match guard.check_fast("POLITICS-2024-1", 5000.0) {
    Decision::Allow => { /* place order */ }
    Decision::Block(reason) => { /* log + skip */ }
}
```

### MarketMomentum

```rust
use pm_core::{MarketMomentum, MomentumContext, MomentumWeights, MomentumResult};

let ctx = MomentumContext {
    volume_24h: 100_000.0,
    volume_7d_avg: 50_000.0,
    spread_pct_now: 0.005,
    spread_pct_1h_ago: 0.020,
    price_1h_ago: 0.55,
    price_now: 0.57,
    whale_fill_price: 0.57,
    num_unique_traders_24h: 15,
};

let result = MomentumResult::compute(&ctx, &MomentumWeights::default());
println!("Momentum: {:.2}", result.momentum_score);
```

### TradeJournal

```rust
use pm_core::{TradeJournal, TradeEntry};

// Real mode (background thread writes CSV)
let journal = TradeJournal::new("/data/trades");

// Noop mode (for testing)
let journal = TradeJournal::noop();

journal.log_trade(TradeEntry {
    timestamp: "2026-02-28T10:15:30Z".to_string(),
    market_id: "POLITICS-2024-1".to_string(),
    whale_address: "0xabcd...".to_string(),
    side: "BUY".to_string(),
    whale_shares: 1000.0,
    whale_price: 0.55,
    my_shares: 100.0,
    my_price: 0.551,
    fill_shares: 100.0,
    fill_price: 0.551,
    slippage_bps: 18.2,
    order_type: "FAK".to_string(),
    status: "FILLED".to_string(),
    resubmit_count: 0,
    block_reason: None,
    latency_ms: 8,
});
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Trading Engine                          │
│   (copy-trading bot, arb bot, etc.)                      │
└────────────┬────────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────────┐
│                     pm-core                             │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐  │
│  │ MarketValidator│  │RiskGuard    │  │MarketMomentum│  │
│  │  (gate)       │  │(circuit brk)│  │ (sentiment) │  │
│  └──────────────┘  └──────────────┘  └─────────────┘  │
│  ┌──────────────┐  ┌──────────────┐                    │
│  │ExposureTracker│  │TradeJournal │                    │
│  │  (PnL)       │  │(async CSV)  │                    │
│  └──────────────┘  └──────────────┘                    │
└─────────────────────────────────────────────────────────┘
```

pm-core is the **decision layer** — it makes the go/no-go calls.
The execution layer (order submission, wallet signing) is handled by your bot.

---

## Documentation

- [market_validator](docs/market-validator.md) — 5-rule binary market gate
- [risk_guard](docs/risk-guard.md) — circuit breaker design
- [market_momentum](docs/market-momentum.md) — 3-factor sentiment score

---

## Status

`pm-core` is actively used on the author's own trading infrastructure (AMD64 Linux, Rust 1.77+). It has ~600 unit tests and integration tests with mock data.

This is a personal project, not a funded product. The API is stable — breaking changes will increment the major version.

## License

MIT — see [LICENSE](LICENSE).
