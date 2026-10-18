/// Exposure tracker — serves whale concentration data to Python scorer.
/// Eliminates journal scan (50-120MB JSONL) from Python scorer hot path.
/// Updated on every FILLED trade; queried via HTTP GET /exposure.
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct ExposureTracker {
    // whale_address (short, 16chars) -> total USD exposure across all markets
    whale_exposures: Arc<RwLock<HashMap<String, f64>>>,
    // market_id -> total USD exposure in that market (for per-market checks)
    market_exposures: Arc<RwLock<HashMap<String, f64>>>,
}

impl ExposureTracker {
    pub fn new() -> Self {
        Self {
            whale_exposures: Arc::new(RwLock::new(HashMap::with_capacity(64))),
            market_exposures: Arc::new(RwLock::new(HashMap::with_capacity(64))),
        }
    }

    /// Call on every FILLED trade — updates exposure maps.
    /// safe to call from any thread.
    pub fn record_fill(&self, market_id: &str, whale_address: &str, shares: f64, price: f64) {
        let cost = shares * price;
        if cost <= 0.0 { return; }

        // Per-whale exposure (short address key, like Python scorer)
        let whale_key = whale_address[..whale_address.len().min(16)].to_string();
        if let Ok(mut w) = self.whale_exposures.write() {
            *w.entry(whale_key).or_insert(0.0) += cost;
        }
        // Per-market exposure
        if let Ok(mut m) = self.market_exposures.write() {
            *m.entry(market_id.to_string()).or_insert(0.0) += cost;
        }
    }

    /// Get whale exposure + total bot exposure + whale concentration.
    /// Queried by Python scorer via HTTP.
    pub fn get_whale_exposure(&self, whale_address: &str) -> (f64, f64, f64) {
        let whale_key = whale_address[..whale_address.len().min(16)].to_string();
        
        let whale_exp = self.whale_exposures.read()
            .ok()
            .and_then(|w| w.get(&whale_key).copied())
            .unwrap_or(0.0);

        let total_exp: f64 = self.whale_exposures.read()
            .map(|w| w.values().sum())
            .unwrap_or(0.0);

        let concentration = if total_exp > 0.0 { whale_exp / total_exp } else { 0.0 };

        (whale_exp, total_exp, concentration)
    }

    /// Full snapshot for JSON serialization (GET /exposure endpoint)
    pub fn snapshot(&self) -> ExposureSnapshot {
        let whale_exposures = self.whale_exposures.read()
            .map(|w| w.clone())
            .unwrap_or_default();
        let market_exposures = self.market_exposures.read()
            .map(|m| m.clone())
            .unwrap_or_default();
        let total: f64 = whale_exposures.values().sum();

        ExposureSnapshot {
            whale_exposures,
            market_exposures,
            total_bot_exposure_usd: total,
        }
    }
}

impl Default for ExposureTracker {
    fn default() -> Self { Self::new() }
}

#[derive(Serialize)]
pub struct ExposureSnapshot {
    /// whale_short_address -> exposure_usd
    #[serde(rename = "whaleExposures")]
    pub whale_exposures: HashMap<String, f64>,
    /// market_id -> exposure_usd
    #[serde(rename = "marketExposures")]
    pub market_exposures: HashMap<String, f64>,
    #[serde(rename = "totalBotExposureUsd")]
    pub total_bot_exposure_usd: f64,
}
