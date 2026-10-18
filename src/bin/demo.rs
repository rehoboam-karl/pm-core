//! demo.rs — shows pm-core capabilities with synthetic data
//!
//! Run: cargo run --bin demo

use pm_core::*;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("=== pm-core Demo ===\n");

    demo_market_validator();
    demo_risk_guard();
    demo_trade_journal();
    demo_market_momentum();
}

/// Shows MarketValidator on synthetic market stats
fn demo_market_validator() {
    println!("--- MarketValidator ---");

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
    match result.pass {
        true => println!("  ✓ Market passes validation"),
        false => println!("  ✗ Rejected: {:?}", result.fail_reason),
    }

    // Test a bad market: wide spread
    let bad_metrics = ValidatorMetrics {
        live: true,
        spread_pct: 15.0,
        book_depth_usd: 200.0,
        hours_remaining: 2.0,
        whale_fill_price: 0.55,
        current_best_ask: 0.60,
    };

    let result = validate(&cfg, &bad_metrics);
    match result.pass {
        true => println!("  ✓ Bad market unexpectedly passed"),
        false => println!("  ✗ Correctly rejected: {:?}", result.fail_reason),
    }
    println!();
}

/// Shows RiskGuard safety decisions
fn demo_risk_guard() {
    println!("--- RiskGuard ---");

    let cfg = RiskGuardConfig::default();
    let mut guard = RiskGuard::new(cfg);

    let result = guard.check_fast("POLITICS-2024-1", 30.0);
    println!("  Small trade (30 shares): {:?}", result.decision);

    let result = guard.check_fast("POLITICS-2024-1", 5000.0);
    println!("  Large trade (5000 shares): {:?}", result.decision);

    let result = guard.check_with_book("POLITICS-2024-1", 5, 100.0);
    println!("  With book depth check: {:?}", result.decision);
    println!();
}

/// Shows TradeJournal (noop mode for demo)
fn demo_trade_journal() {
    println!("--- TradeJournal ---");

    let journal = TradeJournal::noop();
    let entry = TradeEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        market_id: "POLITICS-2024".to_string(),
        whale_address: "0xABCDEF".to_string(),
        side: "BUY".to_string(),
        whale_shares: 100.0,
        whale_price: 0.45,
        my_shares: 0.0,
        my_price: 0.0,
        fill_shares: 0.0,
        fill_price: 0.0,
        slippage_bps: 0.0,
        order_type: "N/A".to_string(),
        status: "BLOCKED".to_string(),
        resubmit_count: 0,
        block_reason: Some("demo".to_string()),
        latency_ms: 0,
    };

    journal.log_trade(entry);
    println!("  Entries written: (noop mode — see real journal in production)");
    println!();
}

/// Shows MarketMomentum on synthetic data
fn demo_market_momentum() {
    println!("--- MarketMomentum ---");

    let weights = MomentumWeights::default();

    // Period 1: baseline
    let ctx = MomentumContext {
        volume_24h: 50_000.0,
        volume_7d_avg: 50_000.0,
        spread_pct_now: 0.01,
        spread_pct_1h_ago: 0.01,
        price_1h_ago: 0.55,
        price_now: 0.55,
        whale_fill_price: 0.55,
        num_unique_traders_24h: 8,
    };
    let result = MomentumResult::compute(&ctx, &weights);
    println!("  Period 1 momentum: {:.1} (calm market)", result.momentum_score);

    // Period 2: volume surge + spread compression = bullish
    let ctx = MomentumContext {
        volume_24h: 100_000.0,
        volume_7d_avg: 50_000.0,
        spread_pct_now: 0.005,
        spread_pct_1h_ago: 0.02,
        price_1h_ago: 0.55,
        price_now: 0.57,
        whale_fill_price: 0.57,
        num_unique_traders_24h: 15,
    };
    let result = MomentumResult::compute(&ctx, &weights);
    println!("  Period 2 momentum: {:.1} (volume surge)", result.momentum_score);

    // Period 3: spread widening = weakening
    let ctx = MomentumContext {
        volume_24h: 20_000.0,
        volume_7d_avg: 50_000.0,
        spread_pct_now: 0.03,
        spread_pct_1h_ago: 0.01,
        price_1h_ago: 0.56,
        price_now: 0.55,
        whale_fill_price: 0.55,
        num_unique_traders_24h: 3,
    };
    let result = MomentumResult::compute(&ctx, &weights);
    println!("  Period 3 momentum: {:.1} (spread widening)", result.momentum_score);
    println!();
}
