use chrono::Utc;
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
/// Trade Journal - Non-blocking trade logging for audit and analysis
/// Logs all trades (filled, partial, failed, blocked) to daily JSONL files
/// Pattern: fire-and-forget via background thread (same as notifier.rs)
use std::sync::mpsc;
use std::thread;

// =============================================================================
// TradeEntry - Complete trade record with all execution details
// =============================================================================

#[derive(Clone, Debug, Serialize)]
pub struct TradeEntry {
    /// ISO 8601 timestamp (e.g., ""2026-02-28T10:15:30Z"")
    pub timestamp: String,
    /// Market/token ID
    pub market_id: String,
    /// Whale wallet address that triggered the copy
    pub whale_address: String,
    /// Trade side: "BUY" or "SELL"
    pub side: String,
    /// Whale's order size (shares)
    pub whale_shares: f64,
    /// Whale's limit price
    pub whale_price: f64,
    /// Our order size (shares)
    pub my_shares: f64,
    /// Our limit price
    pub my_price: f64,
    /// Actual filled shares (from OrderResponse)
    pub fill_shares: f64,
    /// Actual filled price (from OrderResponse)
    pub fill_price: f64,
    /// Slippage in basis points: (fill_price - whale_price) / whale_price * 10000
    pub slippage_bps: f64,
    /// Order type: "FAK" (fill-or-kill) or "GTD" (good-til-date)
    pub order_type: String,
    /// Trade status: "FILLED", "PARTIAL", "FAILED", "BLOCKED"
    pub status: String,
    /// Number of resubmit attempts (0 for first attempt)
    pub resubmit_count: u8,
    /// Why the trade was blocked (if status == "BLOCKED")
    pub block_reason: Option<String>,
    /// Latency from whale event to order submission (milliseconds)
    pub latency_ms: u64,
}

impl TradeEntry {
    /// Calculate slippage in basis points
    pub fn calculate_slippage(fill_price: f64, whale_price: f64) -> f64 {
        if whale_price == 0.0 {
            return 0.0;
        }
        ((fill_price - whale_price) / whale_price) * 10000.0
    }
}

// =============================================================================
// TradeJournal - Non-blocking trade logger
// =============================================================================

#[derive(Clone)]
pub struct TradeJournal {
    tx: mpsc::Sender<TradeEntry>,
}

/// Parameters for logging a blocked trade (avoids 8-param function)
#[derive(Clone, Debug)]
pub struct BlockedTrade {
    pub market_id: String,
    pub whale_address: String,
    pub side: String,
    pub reason: String,
    pub whale_shares: f64,
    pub whale_price: f64,
    pub latency_ms: u64,
}

impl TradeJournal {
    /// Create a new TradeJournal. Spawns a background thread for logging.
    /// data_dir: path to root data directory (e.g., "/data")
    pub fn new(data_dir: &str) -> Self {
        let (tx, rx) = mpsc::channel::<TradeEntry>();
        let data_path = PathBuf::from(data_dir);

        thread::Builder::new()
            .name("trade-journal".into())
            .spawn(move || {
                journal_worker(rx, data_path);
            })
            .expect("Failed to spawn trade journal thread");

        Self { tx }
    }

    /// Create a no-op journal (drops all entries, for testing)
    pub fn noop() -> Self {
        let (tx, _rx) = mpsc::channel::<TradeEntry>();
        Self { tx }
    }

    /// Log a trade entry (non-blocking, fire-and-forget)
    #[inline]
    pub fn log_trade(&self, entry: TradeEntry) {
        let _ = self.tx.send(entry);
    }

    /// Convenience: log a blocked trade
    pub fn log_block(&self, block: BlockedTrade) {
        let entry = TradeEntry {
            timestamp: Utc::now().to_rfc3339(),
            market_id: block.market_id,
            whale_address: block.whale_address,
            side: block.side,
            whale_shares: block.whale_shares,
            whale_price: block.whale_price,
            my_shares: 0.0,
            my_price: 0.0,
            fill_shares: 0.0,
            fill_price: 0.0,
            slippage_bps: 0.0,
            order_type: "N/A".to_string(),
            status: "BLOCKED".to_string(),
            resubmit_count: 0,
            block_reason: Some(block.reason),
            latency_ms: block.latency_ms,
        };
        self.log_trade(entry);
    }
}

// =============================================================================
// Background Worker
// =============================================================================

fn journal_worker(rx: mpsc::Receiver<TradeEntry>, data_dir: PathBuf) {
    // Ensure /data/trades/ directory exists
    let trades_dir = data_dir.join("trades");
    if let Err(e) = fs::create_dir_all(&trades_dir) {
        eprintln!("❌ Failed to create trades directory: {}", e);
        return;
    }

    while let Ok(entry) = rx.recv() {
        // Determine filename from timestamp (YYYY-MM-DD.jsonl)
        let date = entry.timestamp.split('T').next().unwrap_or("unknown");
        let filename = format!("{}.jsonl", date);
        let filepath = trades_dir.join(filename);

        // Append JSON line to daily file
        match OpenOptions::new().create(true).append(true).open(&filepath) {
            Ok(mut file) => match serde_json::to_string(&entry) {
                Ok(json) => {
                    if let Err(e) = writeln!(file, "{}", json) {
                        eprintln!("⚠️ Failed to write trade journal: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to serialize trade entry: {}", e);
                }
            },
            Err(e) => {
                eprintln!("⚠️ Failed to open trade journal file: {}", e);
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_calculate_slippage() {
//         // BUY: positive slippage means we paid more
//         assert_eq!(TradeEntry::calculate_slippage(0.55, 0.50), 1000.0); // +10%
//
//         // SELL: negative slippage means we received less
//         assert_eq!(TradeEntry::calculate_slippage(0.45, 0.50), -1000.0); // -10%
//
//         // No slippage
//         assert_eq!(TradeEntry::calculate_slippage(0.50, 0.50), 0.0);
//
//         // Edge case: zero whale_price
//         assert_eq!(TradeEntry::calculate_slippage(0.50, 0.0), 0.0);
//     }
//
//     #[test]
//     fn test_trade_entry_serialization() {
//         let entry = TradeEntry {
//             timestamp: "2026-02-28T10:15:30Z".to_string(),
//             market_id: 0xabc123.to_string(),
//             whale_address: "0xwhale".to_string(),
//             side: "BUY".to_string(),
//             whale_shares: 1000.0,
//             whale_price: 0.50,
//             my_shares: 500.0,
//             my_price: 0.51,
//             fill_shares: 500.0,
//             fill_price: 0.51,
//             slippage_bps: 200.0,
//             order_type: "FAK".to_string(),
//             status: "FILLED".to_string(),
//             resubmit_count: 0,
//             block_reason: None,
//             latency_ms: 125,
//         };
//
//         let json = serde_json::to_string(&entry).unwrap();
//         assert!(json.contains(""market_id":"0xabc123"));
//         assert!(json.contains(""status":"FILLED"));
//         assert!(json.contains(""slippage_bps":200.0));
//     }
//
//     #[test]
//     fn test_noop_journal_doesnt_panic() {
//         let journal = TradeJournal::noop();
//         let entry = TradeEntry {
//             timestamp: Utc::now().to_rfc3339(),
//             market_id: "test".to_string(),
//             whale_address: "whale_test".to_string().to_string(),
//             side: "BUY".to_string(),
//             whale_shares: 100.0,
//             whale_price: 0.50,
//             my_shares: 50.0,
//             my_price: 0.51,
//             fill_shares: 50.0,
//             fill_price: 0.51,
//             slippage_bps: 200.0,
//             order_type: "FAK".to_string(),
//             status: "FILLED".to_string(),
//             resubmit_count: 0,
//             block_reason: None,
//             latency_ms: 100,
//         };
//         journal.log_trade(entry);
//         journal.log_block("test", "market_test", "BUY", "reason", 100.0, 0.50, 100);
//     }
// }
