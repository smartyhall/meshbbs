//! Minimal metrics scaffolding (Phase 3)
//! This will later be extended with Prometheus exposition and histograms.
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

static RELIABLE_SENT: AtomicU64 = AtomicU64::new(0);
static RELIABLE_ACKED: AtomicU64 = AtomicU64::new(0);
static RELIABLE_FAILED: AtomicU64 = AtomicU64::new(0);
static RELIABLE_RETRIES: AtomicU64 = AtomicU64::new(0);
static ACK_LATENCY_SUM_MS: AtomicU64 = AtomicU64::new(0);
static ACK_LATENCY_COUNT: AtomicU64 = AtomicU64::new(0);
static BROADCAST_ACK_CONFIRMED: AtomicU64 = AtomicU64::new(0);
static BROADCAST_ACK_EXPIRED: AtomicU64 = AtomicU64::new(0);

static GAME_COUNTERS: OnceLock<Mutex<HashMap<String, GameCounter>>> = OnceLock::new();

#[allow(dead_code)]
pub fn inc_reliable_sent() {
    RELIABLE_SENT.fetch_add(1, Ordering::Relaxed);
}
#[allow(dead_code)]
pub fn inc_reliable_acked() {
    RELIABLE_ACKED.fetch_add(1, Ordering::Relaxed);
}
#[allow(dead_code)]
pub fn inc_reliable_failed() {
    RELIABLE_FAILED.fetch_add(1, Ordering::Relaxed);
}
#[allow(dead_code)]
pub fn inc_reliable_retries() {
    RELIABLE_RETRIES.fetch_add(1, Ordering::Relaxed);
}
#[allow(dead_code)]
pub fn observe_ack_latency(sent_at: Instant) {
    let ms = sent_at.elapsed().as_millis() as u64;
    ACK_LATENCY_SUM_MS.fetch_add(ms, Ordering::Relaxed);
    ACK_LATENCY_COUNT.fetch_add(1, Ordering::Relaxed);
}

#[allow(dead_code)]
pub fn inc_broadcast_ack_confirmed() {
    BROADCAST_ACK_CONFIRMED.fetch_add(1, Ordering::Relaxed);
}
#[allow(dead_code)]
pub fn inc_broadcast_ack_expired() {
    BROADCAST_ACK_EXPIRED.fetch_add(1, Ordering::Relaxed);
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GameCounter {
    pub entries: u64,
    pub exits: u64,
    pub currently_active: u64,
    pub concurrent_peak: u64,
}

fn game_counter_lock() -> &'static Mutex<HashMap<String, GameCounter>> {
    GAME_COUNTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn record_game_entry(slug: &str) -> GameCounter {
    let mut guard = game_counter_lock()
        .lock()
        .expect("game counter mutex poisoned");
    let counter = guard.entry(slug.to_string()).or_default();
    counter.entries = counter.entries.saturating_add(1);
    counter.currently_active = counter.currently_active.saturating_add(1);
    if counter.currently_active > counter.concurrent_peak {
        counter.concurrent_peak = counter.currently_active;
    }
    *counter
}

pub fn record_game_exit(slug: &str) -> GameCounter {
    let mut guard = game_counter_lock()
        .lock()
        .expect("game counter mutex poisoned");
    let counter = guard.entry(slug.to_string()).or_default();
    counter.exits = counter.exits.saturating_add(1);
    if counter.currently_active > 0 {
        counter.currently_active -= 1;
    }
    *counter
}

pub fn game_counters_snapshot() -> HashMap<String, GameCounter> {
    game_counter_lock()
        .lock()
        .expect("game counter mutex poisoned")
        .clone()
}

#[cfg(test)]
pub(crate) fn reset_game_counters_for_tests() {
    if let Some(lock) = GAME_COUNTERS.get() {
        let mut guard = lock.lock().expect("game counter mutex poisoned");
        guard.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_entry_exit_updates_counters() {
        reset_game_counters_for_tests();
        assert!(game_counters_snapshot().is_empty());

        let entry_stats = record_game_entry("tinyhack");
        assert_eq!(entry_stats.entries, 1);
        assert_eq!(entry_stats.currently_active, 1);
        assert_eq!(entry_stats.concurrent_peak, 1);
        assert_eq!(entry_stats.exits, 0);

        let mid_snapshot = game_counters_snapshot();
        let tinyhack = mid_snapshot.get("tinyhack").expect("tinyhack counter");
        assert_eq!(tinyhack.entries, 1);
        assert_eq!(tinyhack.currently_active, 1);
        assert_eq!(tinyhack.concurrent_peak, 1);

        let exit_stats = record_game_exit("tinyhack");
        assert_eq!(exit_stats.exits, 1);
        assert_eq!(exit_stats.currently_active, 0);
        assert_eq!(exit_stats.concurrent_peak, 1);

        let final_snapshot = game_counters_snapshot();
        let tinyhack_final = final_snapshot.get("tinyhack").expect("tinyhack counter");
        assert_eq!(tinyhack_final.entries, 1);
        assert_eq!(tinyhack_final.exits, 1);
        assert_eq!(tinyhack_final.currently_active, 0);
    }
}

#[derive(Debug, Default, Clone)]
#[allow(dead_code)] // Fields read primarily in tests / future metrics endpoint
pub struct Snapshot {
    pub reliable_sent: u64,
    pub reliable_acked: u64,
    pub reliable_failed: u64,
    pub reliable_retries: u64,
    pub ack_latency_avg_ms: Option<u64>,
    pub broadcast_ack_confirmed: u64,
    pub broadcast_ack_expired: u64,
}

#[allow(dead_code)]
pub fn snapshot() -> Snapshot {
    let sent = RELIABLE_SENT.load(Ordering::Relaxed);
    let acked = RELIABLE_ACKED.load(Ordering::Relaxed);
    let failed = RELIABLE_FAILED.load(Ordering::Relaxed);
    let retries = RELIABLE_RETRIES.load(Ordering::Relaxed);
    let sum = ACK_LATENCY_SUM_MS.load(Ordering::Relaxed);
    let count = ACK_LATENCY_COUNT.load(Ordering::Relaxed);
    let bcast_ok = BROADCAST_ACK_CONFIRMED.load(Ordering::Relaxed);
    let bcast_exp = BROADCAST_ACK_EXPIRED.load(Ordering::Relaxed);
    Snapshot {
        reliable_sent: sent,
        reliable_acked: acked,
        reliable_failed: failed,
        reliable_retries: retries,
        ack_latency_avg_ms: if count > 0 { Some(sum / count) } else { None },
        broadcast_ack_confirmed: bcast_ok,
        broadcast_ack_expired: bcast_exp,
    }
}
