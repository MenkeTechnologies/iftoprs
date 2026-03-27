use std::collections::VecDeque;
use std::time::Instant;

/// Number of 1-second slots to keep for averaging.
const HISTORY_SLOTS: usize = 40;

/// Per-flow bandwidth history with sliding-window averages.
#[derive(Debug, Clone)]
pub struct FlowHistory {
    /// Bytes sent per 1-second slot (most recent at back).
    pub sent: VecDeque<u64>,
    /// Bytes received per 1-second slot (most recent at back).
    pub recv: VecDeque<u64>,
    /// Cumulative bytes sent.
    pub total_sent: u64,
    /// Cumulative bytes received.
    pub total_recv: u64,
    /// Peak send rate (bytes/sec).
    pub peak_sent: f64,
    /// Peak recv rate (bytes/sec).
    pub peak_recv: f64,
    /// When this flow was last active.
    pub last_seen: Instant,
    /// Process name (if known).
    pub process_name: Option<String>,
    /// Process ID (if known).
    pub pid: Option<u32>,
}

impl FlowHistory {
    pub fn new() -> Self {
        let now = Instant::now();
        let mut sent = VecDeque::with_capacity(HISTORY_SLOTS);
        let mut recv = VecDeque::with_capacity(HISTORY_SLOTS);
        sent.push_back(0);
        recv.push_back(0);
        FlowHistory {
            sent,
            recv,
            total_sent: 0,
            total_recv: 0,
            peak_sent: 0.0,
            peak_recv: 0.0,
            last_seen: now,
            process_name: None,
            pid: None,
        }
    }

    /// Add bytes to the current (most recent) slot.
    pub fn add_sent(&mut self, bytes: u64) {
        self.total_sent += bytes;
        self.last_seen = Instant::now();
        if let Some(slot) = self.sent.back_mut() {
            *slot += bytes;
        }
    }

    pub fn add_recv(&mut self, bytes: u64) {
        self.total_recv += bytes;
        self.last_seen = Instant::now();
        if let Some(slot) = self.recv.back_mut() {
            *slot += bytes;
        }
    }

    /// Rotate: push a new empty slot, evict oldest if > HISTORY_SLOTS.
    pub fn rotate(&mut self) {
        // Update peaks from the slot that just completed
        if let Some(&last) = self.sent.back() {
            let rate = last as f64;
            if rate > self.peak_sent {
                self.peak_sent = rate;
            }
        }
        if let Some(&last) = self.recv.back() {
            let rate = last as f64;
            if rate > self.peak_recv {
                self.peak_recv = rate;
            }
        }

        self.sent.push_back(0);
        self.recv.push_back(0);
        if self.sent.len() > HISTORY_SLOTS {
            self.sent.pop_front();
        }
        if self.recv.len() > HISTORY_SLOTS {
            self.recv.pop_front();
        }
    }

    /// Total bytes transferred over the last `n` seconds.
    fn window_total(slots: &VecDeque<u64>, n: usize) -> f64 {
        let len = slots.len();
        if len == 0 {
            return 0.0;
        }
        let take = n.min(len);
        let sum: u64 = slots.iter().rev().take(take).sum();
        sum as f64
    }

    pub fn avg_sent_2s(&self) -> f64 {
        Self::window_total(&self.sent, 2)
    }
    pub fn avg_sent_10s(&self) -> f64 {
        Self::window_total(&self.sent, 10)
    }
    pub fn avg_sent_40s(&self) -> f64 {
        Self::window_total(&self.sent, 40)
    }

    pub fn avg_recv_2s(&self) -> f64 {
        Self::window_total(&self.recv, 2)
    }
    pub fn avg_recv_10s(&self) -> f64 {
        Self::window_total(&self.recv, 10)
    }
    pub fn avg_recv_40s(&self) -> f64 {
        Self::window_total(&self.recv, 40)
    }
}
