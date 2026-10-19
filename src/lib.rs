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

pub mod exposure_tracker;
pub mod market_cache;
pub mod market_momentum;
pub mod market_validator;
pub mod models;
pub mod risk_guard;
pub mod trade_journal;

// Re-export the key types for ergonomic use
pub use market_validator::validate;
pub use market_validator::{
    ValidatorConfig, ValidatorFailReason, ValidatorMetrics, ValidatorResult,
}; // free function

pub use market_momentum::{
    MomentumContext, MomentumRawMetrics, MomentumResult, MomentumSubScores, MomentumWeights,
};
pub use risk_guard::{
    calc_liquidity_depth, RiskGuard, RiskGuardConfig, SafetyDecision, SafetyEvaluation,
    SafetyReason, TradeSide,
};

pub use exposure_tracker::ExposureTracker;

pub use trade_journal::{TradeEntry, TradeJournal};

pub use market_cache::{CacheLoadResult, CacheStats, MarketCaches};
