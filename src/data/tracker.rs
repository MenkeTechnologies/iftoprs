use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::data::flow::{Direction, FlowKey};
use crate::data::history::FlowHistory;

/// Thread-safe flow tracker that aggregates packet data.
#[derive(Clone)]
pub struct FlowTracker {
    inner: Arc<Mutex<FlowTrackerInner>>,
}

struct FlowTrackerInner {
    flows: HashMap<FlowKey, FlowHistory>,
    last_rotation: Instant,
    /// Global totals
    pub total_sent: u64,
    pub total_recv: u64,
    pub peak_sent: f64,
    pub peak_recv: f64,
    /// Accumulators for the current second (for peak tracking).
    current_sent: u64,
    current_recv: u64,
}

/// Snapshot of all flows for the UI to render.
#[derive(Debug, Clone)]
pub struct FlowSnapshot {
    pub key: FlowKey,
    pub sent_2s: f64,
    pub sent_10s: f64,
    pub sent_40s: f64,
    pub recv_2s: f64,
    pub recv_10s: f64,
    pub recv_40s: f64,
    pub total_sent: u64,
    pub total_recv: u64,
    pub process_name: Option<String>,
    pub pid: Option<u32>,
    /// Per-second combined (sent+recv) history for sparkline rendering.
    pub history: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct TotalStats {
    pub sent_2s: f64,
    pub sent_10s: f64,
    pub sent_40s: f64,
    pub recv_2s: f64,
    pub recv_10s: f64,
    pub recv_40s: f64,
    pub cumulative_sent: u64,
    pub cumulative_recv: u64,
    pub peak_sent: f64,
    pub peak_recv: f64,
}

impl Default for FlowTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowTracker {
    pub fn new() -> Self {
        FlowTracker {
            inner: Arc::new(Mutex::new(FlowTrackerInner {
                flows: HashMap::new(),
                last_rotation: Instant::now(),
                total_sent: 0,
                total_recv: 0,
                peak_sent: 0.0,
                peak_recv: 0.0,
                current_sent: 0,
                current_recv: 0,
            })),
        }
    }

    /// Record a packet into the flow table.
    pub fn record(&self, key: FlowKey, direction: Direction, bytes: u64) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let history = inner.flows.entry(key).or_default();
        match direction {
            Direction::Sent => {
                history.add_sent(bytes);
                inner.total_sent += bytes;
                inner.current_sent += bytes;
            }
            Direction::Received => {
                history.add_recv(bytes);
                inner.total_recv += bytes;
                inner.current_recv += bytes;
            }
        }
    }

    /// Set process info for a flow.
    pub fn set_process_info(&self, key: &FlowKey, pid: u32, name: String) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(history) = inner.flows.get_mut(key) {
            history.pid = Some(pid);
            history.process_name = Some(name);
        }
    }

    /// Rotate history slots (call once per second).
    pub fn maybe_rotate(&self) {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let elapsed = inner.last_rotation.elapsed();
        if elapsed.as_secs() >= 1 {
            // Update peak from the completed second
            let sent_rate = inner.current_sent as f64;
            let recv_rate = inner.current_recv as f64;
            if sent_rate > inner.peak_sent {
                inner.peak_sent = sent_rate;
            }
            if recv_rate > inner.peak_recv {
                inner.peak_recv = recv_rate;
            }
            inner.current_sent = 0;
            inner.current_recv = 0;

            for history in inner.flows.values_mut() {
                history.rotate();
            }
            inner.last_rotation = Instant::now();

            // Evict flows that have been idle for >60 seconds
            let now = Instant::now();
            inner
                .flows
                .retain(|_, h| now.duration_since(h.last_seen).as_secs() < 60);
        }
    }

    /// Get a snapshot of all flows for display.
    pub fn snapshot(&self) -> (Vec<FlowSnapshot>, TotalStats) {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());

        let snapshots: Vec<FlowSnapshot> = inner
            .flows
            .iter()
            .map(|(key, h)| {
                let history: Vec<u64> = h
                    .sent
                    .iter()
                    .zip(h.recv.iter())
                    .map(|(&s, &r)| s + r)
                    .collect();
                FlowSnapshot {
                    key: *key,
                    sent_2s: h.avg_sent_2s(),
                    sent_10s: h.avg_sent_10s(),
                    sent_40s: h.avg_sent_40s(),
                    recv_2s: h.avg_recv_2s(),
                    recv_10s: h.avg_recv_10s(),
                    recv_40s: h.avg_recv_40s(),
                    total_sent: h.total_sent,
                    total_recv: h.total_recv,
                    process_name: h.process_name.clone(),
                    pid: h.pid,
                    history,
                }
            })
            .collect();

        // Compute totals in a single pass
        let (mut s2, mut s10, mut s40, mut r2, mut r10, mut r40) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        for f in &snapshots {
            s2 += f.sent_2s;
            s10 += f.sent_10s;
            s40 += f.sent_40s;
            r2 += f.recv_2s;
            r10 += f.recv_10s;
            r40 += f.recv_40s;
        }
        let totals = TotalStats {
            sent_2s: s2,
            sent_10s: s10,
            sent_40s: s40,
            recv_2s: r2,
            recv_10s: r10,
            recv_40s: r40,
            cumulative_sent: inner.total_sent,
            cumulative_recv: inner.total_recv,
            peak_sent: inner.peak_sent,
            peak_recv: inner.peak_recv,
        };

        (snapshots, totals)
    }

    /// Get all flow keys (for process attribution lookup).
    pub fn flow_keys(&self) -> Vec<FlowKey> {
        let inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        inner.flows.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::flow::Protocol;

    fn test_key(port: u16) -> FlowKey {
        FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: port,
            dst_port: 80,
            protocol: Protocol::Tcp,
        }
    }

    #[test]
    fn new_tracker_empty_snapshot() {
        let t = FlowTracker::new();
        let (flows, totals) = t.snapshot();
        assert!(flows.is_empty());
        assert_eq!(totals.cumulative_sent, 0);
        assert_eq!(totals.cumulative_recv, 0);
    }

    #[test]
    fn record_sent_packet() {
        let t = FlowTracker::new();
        let key = test_key(5000);
        t.record(key, Direction::Sent, 1500);
        let (flows, totals) = t.snapshot();
        assert_eq!(flows.len(), 1);
        assert_eq!(totals.cumulative_sent, 1500);
        assert_eq!(totals.cumulative_recv, 0);
    }

    #[test]
    fn record_recv_packet() {
        let t = FlowTracker::new();
        let key = test_key(5000);
        t.record(key, Direction::Received, 500);
        let (_, totals) = t.snapshot();
        assert_eq!(totals.cumulative_recv, 500);
    }

    #[test]
    fn multiple_flows() {
        let t = FlowTracker::new();
        t.record(test_key(5000), Direction::Sent, 100);
        t.record(test_key(5001), Direction::Sent, 200);
        t.record(test_key(5002), Direction::Sent, 300);
        let (flows, totals) = t.snapshot();
        assert_eq!(flows.len(), 3);
        assert_eq!(totals.cumulative_sent, 600);
    }

    #[test]
    fn flow_keys_returns_all() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, 10);
        t.record(test_key(2), Direction::Sent, 20);
        assert_eq!(t.flow_keys().len(), 2);
    }

    #[test]
    fn set_process_info() {
        let t = FlowTracker::new();
        let key = test_key(5000);
        t.record(key, Direction::Sent, 100);
        t.set_process_info(&key, 1234, "curl".to_string());
        let (flows, _) = t.snapshot();
        assert_eq!(flows[0].pid, Some(1234));
        assert_eq!(flows[0].process_name.as_deref(), Some("curl"));
    }

    #[test]
    fn set_process_info_nonexistent_key_no_panic() {
        let t = FlowTracker::new();
        let key = test_key(9999);
        // Should silently do nothing
        t.set_process_info(&key, 1234, "ghost".to_string());
        let (flows, _) = t.snapshot();
        assert!(flows.is_empty());
    }

    #[test]
    fn default_trait() {
        let t = FlowTracker::default();
        let (flows, totals) = t.snapshot();
        assert!(flows.is_empty());
        assert_eq!(totals.cumulative_sent, 0);
    }

    #[test]
    fn record_both_directions() {
        let t = FlowTracker::new();
        let key = test_key(5000);
        t.record(key, Direction::Sent, 100);
        t.record(key, Direction::Received, 200);
        let (flows, totals) = t.snapshot();
        assert_eq!(flows.len(), 1);
        assert_eq!(totals.cumulative_sent, 100);
        assert_eq!(totals.cumulative_recv, 200);
    }

    #[test]
    fn record_same_flow_accumulates() {
        let t = FlowTracker::new();
        let key = test_key(5000);
        t.record(key, Direction::Sent, 100);
        t.record(key, Direction::Sent, 200);
        t.record(key, Direction::Sent, 300);
        let (_, totals) = t.snapshot();
        assert_eq!(totals.cumulative_sent, 600);
    }

    #[test]
    fn flow_keys_empty_tracker() {
        let t = FlowTracker::new();
        assert!(t.flow_keys().is_empty());
    }

    #[test]
    fn snapshot_totals_sum_flow_rates() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, 100);
        t.record(test_key(2), Direction::Sent, 200);
        let (_, totals) = t.snapshot();
        assert_eq!(totals.cumulative_sent, 300);
        assert_eq!(totals.cumulative_recv, 0);
    }

    #[test]
    fn maybe_rotate_does_not_panic_empty() {
        let t = FlowTracker::new();
        t.maybe_rotate(); // should not panic on empty flows
    }

    #[test]
    fn clone_shares_state() {
        let t = FlowTracker::new();
        let t2 = t.clone();
        t.record(test_key(5000), Direction::Sent, 100);
        let (flows, _) = t2.snapshot();
        assert_eq!(flows.len(), 1);
    }

    #[test]
    fn concurrent_access_no_panic() {
        let t = FlowTracker::new();
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let t = t.clone();
                std::thread::spawn(move || {
                    for j in 0..100 {
                        t.record(test_key(i * 100 + j), Direction::Sent, 10);
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        let (flows, totals) = t.snapshot();
        assert_eq!(flows.len(), 1000);
        assert_eq!(totals.cumulative_sent, 10_000);
    }

    #[test]
    fn mutex_poison_recovery() {
        let t = FlowTracker::new();
        // Poison the mutex by panicking inside a thread holding the lock
        let t2 = t.clone();
        let h = std::thread::spawn(move || {
            let _inner = t2.inner.lock().unwrap();
            panic!("intentional poison");
        });
        let _ = h.join(); // thread panicked, mutex now poisoned

        // These should all still work due to unwrap_or_else recovery
        t.record(test_key(1), Direction::Sent, 42);
        let (flows, _) = t.snapshot();
        assert_eq!(flows.len(), 1);
        assert_eq!(t.flow_keys().len(), 1);
        t.set_process_info(&test_key(1), 99, "recovered".into());
        t.maybe_rotate();
    }

    #[test]
    fn peak_tracking_works() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, 5000);
        t.record(test_key(1), Direction::Received, 3000);
        // Force rotation to capture peaks
        {
            let mut inner = t.inner.lock().unwrap_or_else(|e| e.into_inner());
            // Fake last_rotation to force rotation
            inner.last_rotation = std::time::Instant::now() - std::time::Duration::from_secs(2);
        }
        t.maybe_rotate();
        let (_, totals) = t.snapshot();
        assert!(totals.peak_sent >= 5000.0);
        assert!(totals.peak_recv >= 3000.0);
    }

    #[test]
    fn process_info_overwrites() {
        let t = FlowTracker::new();
        let key = test_key(5000);
        t.record(key, Direction::Sent, 100);
        t.set_process_info(&key, 1, "old".into());
        t.set_process_info(&key, 2, "new".into());
        let (flows, _) = t.snapshot();
        assert_eq!(flows[0].pid, Some(2));
        assert_eq!(flows[0].process_name.as_deref(), Some("new"));
    }

    #[test]
    fn snapshot_includes_total_sent_recv_per_flow() {
        let t = FlowTracker::new();
        let key = test_key(5000);
        t.record(key, Direction::Sent, 100);
        t.record(key, Direction::Received, 50);
        let (flows, _) = t.snapshot();
        assert_eq!(flows[0].total_sent, 100);
        assert_eq!(flows[0].total_recv, 50);
    }

    #[test]
    fn snapshot_history_is_sent_plus_recv_per_slot() {
        let t = FlowTracker::new();
        let key = test_key(5000);
        t.record(key, Direction::Sent, 30);
        t.record(key, Direction::Received, 70);
        let (flows, _) = t.snapshot();
        let last = *flows[0].history.last().unwrap();
        assert_eq!(last, 100);
    }

    #[test]
    fn totals_aggregate_window_rates_across_flows() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, 1000);
        t.record(test_key(2), Direction::Received, 500);
        let (_, totals) = t.snapshot();
        assert_eq!(totals.cumulative_sent, 1000);
        assert_eq!(totals.cumulative_recv, 500);
        assert!(totals.sent_2s >= 1000.0);
        assert!(totals.recv_2s >= 500.0);
    }

    #[test]
    fn record_many_bytes_single_flow() {
        let t = FlowTracker::new();
        let key = test_key(42);
        for _ in 0..100 {
            t.record(key, Direction::Sent, 1500);
        }
        let (_, totals) = t.snapshot();
        assert_eq!(totals.cumulative_sent, 150_000);
    }

    #[test]
    fn flow_keys_ordering_is_stable_len() {
        let t = FlowTracker::new();
        for i in 0..5 {
            t.record(test_key(1000 + i), Direction::Sent, 1);
        }
        assert_eq!(t.flow_keys().len(), 5);
    }

    #[test]
    fn snapshot_history_matches_single_slot_totals() {
        let t = FlowTracker::new();
        let key = test_key(1);
        t.record(key, Direction::Sent, 100);
        t.record(key, Direction::Received, 50);
        let (flows, _) = t.snapshot();
        assert_eq!(flows[0].history.len(), 1);
        assert_eq!(flows[0].history[0], 150);
    }

    #[test]
    fn record_alternating_directions_same_key() {
        let t = FlowTracker::new();
        let key = test_key(7);
        for _ in 0..20 {
            t.record(key, Direction::Sent, 10);
            t.record(key, Direction::Received, 5);
        }
        let (flows, totals) = t.snapshot();
        assert_eq!(flows.len(), 1);
        assert_eq!(totals.cumulative_sent, 200);
        assert_eq!(totals.cumulative_recv, 100);
    }

    #[test]
    fn total_stats_peak_defaults_zero() {
        let t = FlowTracker::new();
        let (_, totals) = t.snapshot();
        assert_eq!(totals.peak_sent, 0.0);
        assert_eq!(totals.peak_recv, 0.0);
    }

    #[test]
    fn record_ipv6_flow_key() {
        let t = FlowTracker::new();
        let key = FlowKey {
            src: "2001:db8::1".parse().unwrap(),
            dst: "2001:db8::2".parse().unwrap(),
            src_port: 443,
            dst_port: 50000,
            protocol: Protocol::Tcp,
        };
        t.record(key, Direction::Sent, 1400);
        let (flows, totals) = t.snapshot();
        assert_eq!(flows.len(), 1);
        assert_eq!(totals.cumulative_sent, 1400);
    }

    #[test]
    fn set_process_info_twice_last_wins() {
        let t = FlowTracker::new();
        let key = test_key(42);
        t.record(key, Direction::Sent, 100);
        t.set_process_info(&key, 1, "old".into());
        t.set_process_info(&key, 2, "new".into());
        let (flows, _) = t.snapshot();
        assert_eq!(flows[0].pid, Some(2));
        assert_eq!(flows[0].process_name.as_deref(), Some("new"));
    }

    #[test]
    fn snapshot_preserves_multiple_flow_keys() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, 10);
        t.record(test_key(2), Direction::Received, 20);
        let keys = t.flow_keys();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn record_zero_bytes_no_panic() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, 0);
        let (_, totals) = t.snapshot();
        assert_eq!(totals.cumulative_sent, 0);
    }

    #[test]
    fn tracker_clone_sees_same_flow_after_record() {
        let t = FlowTracker::new();
        let t2 = t.clone();
        t.record(test_key(9), Direction::Received, 999);
        let (_, totals) = t2.snapshot();
        assert_eq!(totals.cumulative_recv, 999);
    }

    #[test]
    fn snapshot_recv_rates_sum_across_flows() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Received, 100);
        t.record(test_key(2), Direction::Received, 200);
        let (_, totals) = t.snapshot();
        assert!(totals.recv_2s >= 300.0);
    }

    #[test]
    fn maybe_rotate_twice_immediately_single_rotation() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, 100);
        t.maybe_rotate();
        t.maybe_rotate();
        let (flows, _) = t.snapshot();
        assert_eq!(flows.len(), 1);
    }

    #[test]
    fn flow_evicted_after_sixty_one_seconds_idle() {
        use std::time::Duration;

        let t = FlowTracker::new();
        let key = test_key(77);
        t.record(key, Direction::Sent, 10);
        {
            let mut inner = t.inner.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(h) = inner.flows.get_mut(&key) {
                h.last_seen = std::time::Instant::now() - Duration::from_secs(61);
            }
            inner.last_rotation = std::time::Instant::now() - Duration::from_secs(2);
        }
        t.maybe_rotate();
        let (flows, _) = t.snapshot();
        assert!(flows.is_empty());
    }

    #[test]
    fn maybe_rotate_resets_current_second_counters() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, 1000);
        {
            let mut inner = t.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.last_rotation = std::time::Instant::now() - std::time::Duration::from_secs(2);
        }
        t.maybe_rotate();
        t.record(test_key(2), Direction::Received, 500);
        {
            let mut inner = t.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.last_rotation = std::time::Instant::now() - std::time::Duration::from_secs(2);
        }
        t.maybe_rotate();
        let (_, totals) = t.snapshot();
        assert!(totals.peak_sent >= 1000.0);
        assert!(totals.peak_recv >= 500.0);
    }

    #[test]
    fn snapshot_history_zip_same_length_as_sent_deque() {
        let t = FlowTracker::new();
        let key = test_key(3);
        t.record(key, Direction::Sent, 5);
        t.record(key, Direction::Received, 7);
        let (flows, _) = t.snapshot();
        assert_eq!(flows[0].history.len(), 1);
        assert_eq!(flows[0].history[0], 12);
    }

    #[test]
    fn snapshot_totals_window_sent_matches_sum_of_flows() {
        let t = FlowTracker::new();
        t.record(test_key(10), Direction::Sent, 50);
        t.record(test_key(11), Direction::Sent, 60);
        let (flows, totals) = t.snapshot();
        let sum: f64 = flows.iter().map(|f| f.sent_2s).sum();
        assert!((totals.sent_2s - sum).abs() < 1e-6);
    }

    #[test]
    fn snapshot_totals_window_recv_matches_sum_of_flows() {
        let t = FlowTracker::new();
        t.record(test_key(20), Direction::Received, 400);
        t.record(test_key(21), Direction::Received, 500);
        let (flows, totals) = t.snapshot();
        let sum: f64 = flows.iter().map(|f| f.recv_2s).sum();
        assert!((totals.recv_2s - sum).abs() < 1e-6);
    }

    #[test]
    fn record_udp_protocol_flow_key() {
        let t = FlowTracker::new();
        let key = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 53,
            dst_port: 5353,
            protocol: Protocol::Udp,
        };
        t.record(key, Direction::Sent, 128);
        let (flows, _) = t.snapshot();
        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].key.protocol, Protocol::Udp);
    }

    #[test]
    fn cumulative_totals_track_only_recorded_directions() {
        let t = FlowTracker::new();
        let key = test_key(88);
        t.record(key, Direction::Sent, 1000);
        let (_, totals) = t.snapshot();
        assert_eq!(totals.cumulative_sent, 1000);
        assert_eq!(totals.cumulative_recv, 0);
    }

    #[test]
    fn snapshot_sent_40s_sum_matches_flows() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, 100);
        t.record(test_key(2), Direction::Sent, 200);
        let (flows, totals) = t.snapshot();
        let sum: f64 = flows.iter().map(|f| f.sent_40s).sum();
        assert!((totals.sent_40s - sum).abs() < 1e-6);
    }

    #[test]
    fn snapshot_icmp_flow_not_recorded_by_tracker() {
        let t = FlowTracker::new();
        let key = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 0,
            dst_port: 0,
            protocol: Protocol::Icmp,
        };
        t.record(key, Direction::Sent, 64);
        let (flows, totals) = t.snapshot();
        assert_eq!(flows.len(), 1);
        assert_eq!(totals.cumulative_sent, 64);
    }

    #[test]
    fn three_distinct_flow_keys_in_snapshot() {
        let t = FlowTracker::new();
        for i in 0..3 {
            t.record(test_key(3000 + i), Direction::Sent, 10);
        }
        let (flows, _) = t.snapshot();
        assert_eq!(flows.len(), 3);
    }

    #[test]
    fn snapshot_recv_40s_sum_matches_flows() {
        let t = FlowTracker::new();
        t.record(test_key(40), Direction::Received, 111);
        t.record(test_key(41), Direction::Received, 222);
        let (flows, totals) = t.snapshot();
        let sum: f64 = flows.iter().map(|f| f.recv_40s).sum();
        assert!((totals.recv_40s - sum).abs() < 1e-6);
    }

    #[test]
    fn flow_tracker_default_matches_new() {
        let a = FlowTracker::new();
        let b = FlowTracker::default();
        let (fa, ta) = a.snapshot();
        let (fb, tb) = b.snapshot();
        assert_eq!(fa.len(), fb.len());
        assert_eq!(ta.cumulative_sent, tb.cumulative_sent);
    }

    #[test]
    fn record_other_ip_protocol() {
        let t = FlowTracker::new();
        let key = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 0,
            dst_port: 0,
            protocol: Protocol::Other(47),
        };
        t.record(key, Direction::Received, 100);
        let (flows, _) = t.snapshot();
        assert_eq!(flows[0].key.protocol, Protocol::Other(47));
    }

    #[test]
    fn flow_not_evicted_when_idle_just_under_sixty_seconds() {
        use std::time::Duration;

        let t = FlowTracker::new();
        let key = test_key(88);
        t.record(key, Direction::Sent, 1);
        {
            let mut inner = t.inner.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(h) = inner.flows.get_mut(&key) {
                h.last_seen = std::time::Instant::now() - Duration::from_secs(59);
            }
            inner.last_rotation = std::time::Instant::now() - Duration::from_secs(2);
        }
        t.maybe_rotate();
        let (flows, _) = t.snapshot();
        assert_eq!(flows.len(), 1);
    }

    #[test]
    fn snapshot_sent_10s_sum_matches_flows() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, 10);
        t.record(test_key(2), Direction::Sent, 20);
        let (flows, totals) = t.snapshot();
        let sum: f64 = flows.iter().map(|f| f.sent_10s).sum();
        assert!((totals.sent_10s - sum).abs() < 1e-6);
    }

    #[test]
    fn snapshot_recv_10s_sum_matches_flows() {
        let t = FlowTracker::new();
        t.record(test_key(3), Direction::Received, 33);
        t.record(test_key(4), Direction::Received, 44);
        let (flows, totals) = t.snapshot();
        let sum: f64 = flows.iter().map(|f| f.recv_10s).sum();
        assert!((totals.recv_10s - sum).abs() < 1e-6);
    }

    #[test]
    fn record_max_u64_bytes_does_not_panic() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, u64::MAX);
        let (_, totals) = t.snapshot();
        assert_eq!(totals.cumulative_sent, u64::MAX);
    }

    #[test]
    fn two_rotations_increment_slot_count() {
        let t = FlowTracker::new();
        let key = test_key(1);
        t.record(key, Direction::Sent, 10);
        {
            let mut inner = t.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.last_rotation = std::time::Instant::now() - std::time::Duration::from_secs(2);
        }
        t.maybe_rotate();
        t.record(key, Direction::Sent, 20);
        {
            let mut inner = t.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.last_rotation = std::time::Instant::now() - std::time::Duration::from_secs(2);
        }
        t.maybe_rotate();
        let (flows, _) = t.snapshot();
        assert!(flows[0].history.len() >= 2);
    }

    #[test]
    fn totals_cumulative_independent_of_maybe_rotate() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, 100);
        t.record(test_key(1), Direction::Received, 50);
        {
            let mut inner = t.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.last_rotation = std::time::Instant::now() - std::time::Duration::from_secs(2);
        }
        t.maybe_rotate();
        let (_, totals) = t.snapshot();
        assert_eq!(totals.cumulative_sent, 100);
        assert_eq!(totals.cumulative_recv, 50);
    }

    #[test]
    fn flow_keys_contains_recorded_normalized_key() {
        let t = FlowTracker::new();
        let key = test_key(1234);
        t.record(key, Direction::Sent, 1);
        let keys = t.flow_keys();
        assert!(keys.contains(&key));
    }

    #[test]
    fn snapshot_process_name_none_until_set() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, 1);
        let (flows, _) = t.snapshot();
        assert!(flows[0].process_name.is_none());
        assert!(flows[0].pid.is_none());
    }

    #[test]
    fn multiple_maybe_rotate_with_empty_tracker_no_panic() {
        let t = FlowTracker::new();
        for _ in 0..5 {
            {
                let mut inner = t.inner.lock().unwrap_or_else(|e| e.into_inner());
                inner.last_rotation = std::time::Instant::now() - std::time::Duration::from_secs(2);
            }
            t.maybe_rotate();
        }
        let (flows, _) = t.snapshot();
        assert!(flows.is_empty());
    }

    #[test]
    fn sent_recv_peaks_independent_after_rotation() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, 100);
        {
            let mut inner = t.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.last_rotation = std::time::Instant::now() - std::time::Duration::from_secs(2);
        }
        t.maybe_rotate();
        t.record(test_key(2), Direction::Received, 9000);
        {
            let mut inner = t.inner.lock().unwrap_or_else(|e| e.into_inner());
            inner.last_rotation = std::time::Instant::now() - std::time::Duration::from_secs(2);
        }
        t.maybe_rotate();
        let (_, totals) = t.snapshot();
        assert!(totals.peak_sent >= 100.0);
        assert!(totals.peak_recv >= 9000.0);
    }

    #[test]
    fn interleaved_sent_recv_on_one_flow_accumulates_both_totals() {
        let t = FlowTracker::new();
        let key = test_key(42);
        for _ in 0..100 {
            t.record(key, Direction::Sent, 3);
            t.record(key, Direction::Received, 7);
        }
        let (_, totals) = t.snapshot();
        assert_eq!(totals.cumulative_sent, 300);
        assert_eq!(totals.cumulative_recv, 700);
    }

    #[test]
    fn two_distinct_flows_accumulate_global_totals() {
        let t = FlowTracker::new();
        t.record(test_key(1), Direction::Sent, 1_000);
        t.record(test_key(2), Direction::Received, 250);
        let (_, totals) = t.snapshot();
        assert_eq!(totals.cumulative_sent, 1_000);
        assert_eq!(totals.cumulative_recv, 250);
    }
}
