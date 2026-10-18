# MarketValidator — Binary Gate for Market Quality

Before placing any order, MarketValidator runs a set of binary checks against
the current market state. If any check fails, the market is rejected.

## Philosophy

MarketValidator is a **binary gate** — markets either pass all checks or get
rejected outright. There is no "almost" or "partially valid." This keeps
the decision logic simple and auditable.

## Checks

| Check | Threshold | Why |
|-------|-----------|-----|
| `live` | must be true | No trading in resolved/halted markets |
| `spread_pct` | < 10% | Wide spread = toxic slippage |
| `book_depth_usd` | > $2,000 | Need enough liquidity to fill |
| `hours_remaining` | > 6h | Don't trade markets about to close |
| `slippage_bps` | < 15 bps | Price impact must be acceptable |

## Usage

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
if !result.pass {
    println!("Rejected: {:?}", result.fail_reason);
}
```

## Metrics collection

The metrics come from your market data layer. Example for Polymarket:

```rust
// Fetch order book
let book = clob_client.book(&token_id).await?;
let spread_pct = ((book.asks[0].price - book.bids[0].price) / book.asks[0].price) * 100.0;
let book_depth_usd = book.bids.iter()
    .take(10)
    .map(|b| b.price * b.size)
    .sum();

// Fetch market info
let market = market_api.market(&token_id).await?;
let hours_remaining = market.end_date - now;
```

## Output

```rust
pub struct ValidationResult {
    pub pass: bool,
    pub fail_reason: Option<String>,  // e.g. "SPREAD_TOO_WIDE"
    pub decision_path: String,        // e.g. "market_validation"
}
```

## Demo

```bash
cargo run --bin demo
```

The demo shows both a passing market and a failing one (wide spread).
