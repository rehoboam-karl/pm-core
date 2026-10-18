//! `pm-core` — Core market validation, risk management and scoring primitives
//! for the Polymarket CLOB trading ecosystem.
//!
//! This library provides production-grade building blocks for trading systems
//! on Polymarket's CLOB API:
//!
//! - **Market Validation** — binary gate that filters lowquality markets before
//!   entry (liquidity, spread, operating hours, slippage)
//! - **Market Momentum** — multi-factor momentum scoring combining volume
//!   acceleration, spread trends and price stability
//! - **Risk Guard** — per-market and global exposure limits with safety
//!   decisions and dynamic depth-adjusted position sizing
//! - **Exposure Tracker** — in-memory multi-market position tracking with
//!   cost-basis P&L and drawdown monitoring
//! - **Trade Journal** — CSV append-only journal with entries structured for
//!   backtesting and audit trails
//! - **Market Cache** — thread-safe price/volume cache with TTL for high-frequency
//!   scoring
//!
//! All modules are framework-agnostic: no runtime, no network I/O, no API
//! credentials required.  Pure business logic with exhaustive unit tests.

pub mod models;
pub mod market_validator;
pub mod market_momentum;
pub mod risk_guard;
pub mod exposure_tracker;
pub mod trade_journal;
pub mod market_cache;

// Re-export the key types for ergonomic use
pub use market_validator::{
    ValidatorConfig, ValidatorMetrics, ValidatorResult, ValidatorFailReason,
};
pub use market_validator::validate; // free function

pub use market_momentum::{
    MomentumContext, MomentumSubScores, MomentumResult, MomentumRawMetrics, MomentumWeights,
};
pub use risk_guard::{
    RiskGuard, RiskGuardConfig, SafetyDecision, SafetyEvaluation, SafetyReason,
    TradeSide, calc_liquidity_depth,
};

pub use exposure_tracker::ExposureTracker;

pub use trade_journal::{TradeJournal, TradeEntry};

pub use market_cache::{MarketCaches, CacheStats, CacheLoadResult};
