# pm-core

> Production-grade market validation, risk management and scoring primitives
> for the Polymarket CLOB trading ecosystem.

`pm-core` is a Rust library that implements the core trading primitives used in
production Polymarket bots — extracted and cleaned for standalone use.

**What it does:**

- **Market Validation** — 5-rule binary gate filters low-quality markets before
  entry (volume, spread, liquidity, operating hours, slippage)
- **Market Momentum** — multi-factor scoring combining volume acceleration,
  spread trends and price stability
- **Risk Guard** — per-market and global exposure limits with dynamic
  depth-adjusted position sizing
- **Exposure Tracker** — in-memory multi-market position tracking with
  cost-basis P&L and drawdown monitoring
- **Trade Journal** — CSV append-only journal with backtesting-ready entries
- **Market Cache** — thread-safe price/volume cache with TTL for high-frequency
  scoring

**Why Rust?** No GC pauses, predictable latency, fearless concurrency.
All modules are framework-agnostic: no runtime, no network I/O, no API
credentials required.

---

## Quick Start

```toml
# Cargo.toml
[dependencies]
pm-core = { git = "https://github.com/fernando-karl/pm-core" }
```

```rust
use pm_core::*;

let cfg = market_validator::ValidatorConfig::default();
let validator = MarketValidator::new(cfg);

let stats = MarketStats {
    volume_24h: 50_000.0,
    spread: 0.02,
    liquidity: 10_000.0,
    open_interest: 5_000.0,
    num_takers: 8,
    hours_since_created: 48,
};

match validator.validate_with_stats("Will X happen by date?", &stats) {
    Ok(_)  => println!("market passes quality gate"),
    Err(e) => println!("rejected: {}", e),
}
```

Run the demo binary:

```bash
cargo run --bin demo
```

---

## Modules

### `market_validator`
Binary pass/fail gate. A market must satisfy ALL rules:

| Rule | Threshold |
|------|-----------|
| Volume | ≥ $1,000 in 24h |
| Spread | ≤ 10% of price |
| Liquidity | ≥ $500 at top of book |
| Age | ≥ 6 hours since creation |
| Slippage | ≤ 15% on FAK simulation |

### `market_momentum`
Composite score 0–100 combining:
- Volume acceleration (3-period trend)
- Spread compression vs baseline
- Price stability (standard deviation of recent prices)

### `risk_guard`
Position sizing with hard limits:
- `max_order_usd` — single order cap
- `max_market_exposure` — per-market cap
- `max_total_exposure` — portfolio-wide cap
- `daily_loss_limit` — automatic circuit breaker

### `exposure_tracker`
Tracks open positions in-memory with:
- Cost-basis P&L per market
- Drawdown monitoring
- Multi-market aggregation

### `trade_journal`
Append-only CSV journal with fields:
`ts, market, side, size, price, order_type, filled, avg_fill_price, cost_usdc, status, tx_hash`

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Trading Bot                                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │ Market   │→ │  Risk    │→ │  Order Executor   │  │
│  │ Validator │  │  Guard   │  │                  │  │
│  └──────────┘  └──────────┘  └──────────────────┘  │
│        ↓             ↓                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐  │
│  │ Market   │  │Exposure │  │  Trade Journal   │  │
│  │ Momentum │  │ Tracker │  │                  │  │
│  └──────────┘  └──────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────┘
                     ↑
              pm-core (this crate)
```

---

## License

MIT — see [LICENSE](LICENSE).
