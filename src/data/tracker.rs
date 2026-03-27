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
        let mut inner = self.inner.lock().unwrap();
        let history = inner.flows.entry(key).or_insert_with(FlowHistory::new);
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
        let mut inner = self.inner.lock().unwrap();
        if let Some(history) = inner.flows.get_mut(key) {
            history.pid = Some(pid);
            history.process_name = Some(name);
        }
    }

    /// Rotate history slots (call once per second).
    pub fn maybe_rotate(&self) {
        let mut inner = self.inner.lock().unwrap();
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
        let inner = self.inner.lock().unwrap();

        let snapshots: Vec<FlowSnapshot> = inner
            .flows
            .iter()
            .map(|(key, h)| FlowSnapshot {
                key: key.clone(),
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
            })
            .collect();

        // Compute totals by summing flow averages
        let totals = TotalStats {
            sent_2s: snapshots.iter().map(|f| f.sent_2s).sum(),
            sent_10s: snapshots.iter().map(|f| f.sent_10s).sum(),
            sent_40s: snapshots.iter().map(|f| f.sent_40s).sum(),
            recv_2s: snapshots.iter().map(|f| f.recv_2s).sum(),
            recv_10s: snapshots.iter().map(|f| f.recv_10s).sum(),
            recv_40s: snapshots.iter().map(|f| f.recv_40s).sum(),
            cumulative_sent: inner.total_sent,
            cumulative_recv: inner.total_recv,
            peak_sent: inner.peak_sent,
            peak_recv: inner.peak_recv,
        };

        (snapshots, totals)
    }

    /// Get all flow keys (for process attribution lookup).
    pub fn flow_keys(&self) -> Vec<FlowKey> {
        let inner = self.inner.lock().unwrap();
        inner.flows.keys().cloned().collect()
    }
}
