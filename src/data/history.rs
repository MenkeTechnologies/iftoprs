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

impl Default for FlowHistory {
    fn default() -> Self {
        Self::new()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_history_is_empty() {
        let h = FlowHistory::new();
        assert_eq!(h.total_sent, 0);
        assert_eq!(h.total_recv, 0);
        assert_eq!(h.peak_sent, 0.0);
        assert_eq!(h.peak_recv, 0.0);
        assert_eq!(h.sent.len(), 1);
        assert_eq!(h.recv.len(), 1);
    }

    #[test]
    fn new_history_sliding_averages_are_zero() {
        let h = FlowHistory::new();
        assert_eq!(h.avg_sent_2s(), 0.0);
        assert_eq!(h.avg_recv_2s(), 0.0);
        assert_eq!(h.avg_sent_40s(), 0.0);
        assert_eq!(h.avg_recv_40s(), 0.0);
    }

    #[test]
    fn add_sent_accumulates() {
        let mut h = FlowHistory::new();
        h.add_sent(100);
        h.add_sent(200);
        assert_eq!(h.total_sent, 300);
        assert_eq!(*h.sent.back().unwrap(), 300);
    }

    #[test]
    fn add_recv_accumulates() {
        let mut h = FlowHistory::new();
        h.add_recv(500);
        assert_eq!(h.total_recv, 500);
        assert_eq!(*h.recv.back().unwrap(), 500);
    }

    #[test]
    fn rotate_pushes_new_slot() {
        let mut h = FlowHistory::new();
        h.add_sent(1000);
        h.rotate();
        assert_eq!(h.sent.len(), 2);
        assert_eq!(*h.sent.back().unwrap(), 0); // new slot is zero
        assert_eq!(h.peak_sent, 1000.0);
    }

    #[test]
    fn rotate_evicts_after_40_slots() {
        let mut h = FlowHistory::new();
        for _ in 0..50 {
            h.add_sent(1);
            h.rotate();
        }
        assert!(h.sent.len() <= 40);
    }

    #[test]
    fn window_averages() {
        let mut h = FlowHistory::new();
        // Slot 0: 100 bytes sent
        h.add_sent(100);
        h.rotate();
        // Slot 1: 200 bytes sent
        h.add_sent(200);

        // 2s window = last 2 slots = 200 (current) + 100 (prev) = 300
        assert_eq!(h.avg_sent_2s(), 300.0);
        // 10s window with only 2 slots = same 300
        assert_eq!(h.avg_sent_10s(), 300.0);
    }

    #[test]
    fn peak_tracking() {
        let mut h = FlowHistory::new();
        h.add_sent(500);
        h.rotate();
        h.add_sent(1000);
        h.rotate();
        h.add_sent(200);
        h.rotate();
        assert_eq!(h.peak_sent, 1000.0);
    }

    #[test]
    fn default_trait() {
        let h = FlowHistory::default();
        assert_eq!(h.total_sent, 0);
        assert_eq!(h.total_recv, 0);
    }

    #[test]
    fn last_seen_updates_on_sent() {
        let h1 = FlowHistory::new();
        let before = h1.last_seen;
        std::thread::sleep(std::time::Duration::from_millis(1));
        let mut h2 = FlowHistory::new();
        h2.add_sent(100);
        assert!(h2.last_seen >= before);
    }

    #[test]
    fn last_seen_updates_on_recv() {
        let mut h = FlowHistory::new();
        let before = h.last_seen;
        std::thread::sleep(std::time::Duration::from_millis(1));
        h.add_recv(100);
        assert!(h.last_seen >= before);
    }

    #[test]
    fn process_fields_none_by_default() {
        let h = FlowHistory::new();
        assert!(h.process_name.is_none());
        assert!(h.pid.is_none());
    }

    #[test]
    fn recv_window_averages() {
        let mut h = FlowHistory::new();
        h.add_recv(100);
        h.rotate();
        h.add_recv(200);
        assert_eq!(h.avg_recv_2s(), 300.0);
        assert_eq!(h.avg_recv_10s(), 300.0);
        assert_eq!(h.avg_recv_40s(), 300.0);
    }

    #[test]
    fn recv_peak_tracking() {
        let mut h = FlowHistory::new();
        h.add_recv(500);
        h.rotate();
        h.add_recv(1000);
        h.rotate();
        assert_eq!(h.peak_recv, 1000.0);
    }

    #[test]
    fn window_avg_single_slot() {
        let mut h = FlowHistory::new();
        h.add_sent(42);
        assert_eq!(h.avg_sent_2s(), 42.0);
        assert_eq!(h.avg_sent_10s(), 42.0);
        assert_eq!(h.avg_sent_40s(), 42.0);
    }

    #[test]
    fn window_avg_many_slots() {
        let mut h = FlowHistory::new();
        for i in 0..20 {
            h.add_sent(i * 10);
            h.rotate();
        }
        // 2s = last 2 slots, which are 0 (new empty) and the value from last iteration
        let s2 = h.avg_sent_2s();
        let s10 = h.avg_sent_10s();
        let s40 = h.avg_sent_40s();
        assert!(s2 <= s10);
        assert!(s10 <= s40);
    }

    #[test]
    fn rotate_evicts_recv_after_40_slots() {
        let mut h = FlowHistory::new();
        for _ in 0..50 {
            h.add_recv(1);
            h.rotate();
        }
        assert!(h.recv.len() <= 40);
    }

    #[test]
    fn add_sent_and_recv_same_slot() {
        let mut h = FlowHistory::new();
        h.add_sent(10);
        h.add_recv(20);
        assert_eq!(h.total_sent, 10);
        assert_eq!(h.total_recv, 20);
        assert_eq!(*h.sent.back().unwrap(), 10);
        assert_eq!(*h.recv.back().unwrap(), 20);
    }

    #[test]
    fn avg_sent_40s_respects_slot_cap() {
        let mut h = FlowHistory::new();
        for i in 0..45 {
            h.add_sent(100 + i);
            h.rotate();
        }
        // Only last 40 non-empty slots contribute; window sums last 40 slots
        let a = h.avg_sent_40s();
        assert!(a > 0.0);
        assert!(a < 45.0 * 200.0);
    }

    #[test]
    fn peak_sent_not_updated_from_empty_slot_after_rotate() {
        let mut h = FlowHistory::new();
        h.add_sent(100);
        h.rotate();
        h.add_sent(0);
        h.rotate();
        assert_eq!(h.peak_sent, 100.0);
    }

    #[test]
    fn rotate_preserves_total_counters() {
        let mut h = FlowHistory::new();
        h.add_sent(50);
        h.add_recv(25);
        h.rotate();
        assert_eq!(h.total_sent, 50);
        assert_eq!(h.total_recv, 25);
    }

    #[test]
    fn recv_peak_tracks_across_rotations() {
        let mut h = FlowHistory::new();
        h.add_recv(10);
        h.rotate();
        h.add_recv(200);
        h.rotate();
        h.add_recv(5);
        assert_eq!(h.peak_recv, 200.0);
    }

    #[test]
    fn sent_and_recv_peaks_independent() {
        let mut h = FlowHistory::new();
        h.add_sent(1000);
        h.rotate();
        h.add_recv(5000);
        h.rotate();
        assert_eq!(h.peak_sent, 1000.0);
        assert_eq!(h.peak_recv, 5000.0);
    }

    #[test]
    fn avg_recv_2s_single_slot_matches_back() {
        let mut h = FlowHistory::new();
        h.add_recv(333);
        assert_eq!(h.avg_recv_2s(), 333.0);
    }

    #[test]
    fn sent_deque_never_exceeds_history_slots_after_many_rotates() {
        let mut h = FlowHistory::new();
        for _ in 0..100 {
            h.add_sent(1);
            h.rotate();
        }
        assert!(h.sent.len() <= 40);
    }

    #[test]
    fn total_sent_accumulates_across_rotations() {
        let mut h = FlowHistory::new();
        h.add_sent(100);
        h.rotate();
        h.add_sent(200);
        assert_eq!(h.total_sent, 300);
    }

    #[test]
    fn recv_deque_bounded_after_rotations() {
        let mut h = FlowHistory::new();
        for _ in 0..60 {
            h.add_recv(1);
            h.rotate();
        }
        assert!(h.recv.len() <= 40);
    }

    #[test]
    fn avg_sent_10s_matches_window_total() {
        let mut h = FlowHistory::new();
        h.add_sent(10);
        h.rotate();
        h.add_sent(20);
        assert_eq!(h.avg_sent_10s(), 30.0);
    }

    #[test]
    fn rotate_exactly_thirty_nine_times_keeps_at_most_forty_slots() {
        let mut h = FlowHistory::new();
        for _ in 0..39 {
            h.add_sent(1);
            h.rotate();
        }
        assert_eq!(h.sent.len(), 40);
        assert_eq!(h.recv.len(), 40);
    }

    #[test]
    fn peak_recv_tracks_after_sent_only_rotations() {
        let mut h = FlowHistory::new();
        h.add_recv(800);
        h.rotate();
        h.add_recv(0);
        h.rotate();
        assert_eq!(h.peak_recv, 800.0);
    }

    #[test]
    fn avg_recv_40s_sums_last_forty_nonzero_slots() {
        let mut h = FlowHistory::new();
        for i in 0..5 {
            h.add_recv(10 * (i + 1));
            h.rotate();
        }
        let sum: u64 = (1..=5).map(|i| 10 * i).sum();
        assert_eq!(h.avg_recv_40s(), sum as f64);
    }

    #[test]
    fn add_sent_after_many_rotates_current_slot_accumulates() {
        let mut h = FlowHistory::new();
        for _ in 0..10 {
            h.rotate();
        }
        h.add_sent(99);
        assert_eq!(*h.sent.back().unwrap(), 99);
        assert_eq!(h.total_sent, 99);
    }

    #[test]
    fn rotate_recv_only_updates_peak_recv_not_sent() {
        let mut h = FlowHistory::new();
        h.add_recv(333);
        h.rotate();
        assert_eq!(h.peak_sent, 0.0);
        assert_eq!(h.peak_recv, 333.0);
    }

    #[test]
    fn total_recv_independent_of_sent_rotations() {
        let mut h = FlowHistory::new();
        h.add_sent(999);
        h.rotate();
        h.add_recv(1);
        assert_eq!(h.total_recv, 1);
        assert_eq!(h.total_sent, 999);
    }

    #[test]
    fn avg_sent_2s_after_three_slots_partial_sum() {
        let mut h = FlowHistory::new();
        h.add_sent(10);
        h.rotate();
        h.add_sent(20);
        h.rotate();
        h.add_sent(100);
        assert_eq!(h.avg_sent_2s(), 120.0);
    }

    #[test]
    fn rotate_records_both_peaks_when_sent_and_recv_in_slot() {
        let mut h = FlowHistory::new();
        h.add_sent(1000);
        h.add_recv(500);
        h.rotate();
        assert_eq!(h.peak_sent, 1000.0);
        assert_eq!(h.peak_recv, 500.0);
    }

    #[test]
    fn avg_recv_10s_matches_sum_over_slots() {
        let mut h = FlowHistory::new();
        h.add_recv(5);
        h.rotate();
        h.add_recv(15);
        assert_eq!(h.avg_recv_10s(), 20.0);
    }

    #[test]
    fn peak_sent_monotonic_across_rotations() {
        let mut h = FlowHistory::new();
        h.add_sent(100);
        h.rotate();
        h.add_sent(200);
        h.rotate();
        h.add_sent(50);
        h.rotate();
        assert_eq!(h.peak_sent, 200.0);
    }

    #[test]
    fn avg_sent_40s_full_window_after_many_slots() {
        let mut h = FlowHistory::new();
        for i in 0..10 {
            h.add_sent((i + 1) * 10);
            h.rotate();
        }
        let sum: u64 = (1..=10).map(|i| i * 10).sum();
        assert_eq!(h.avg_sent_40s(), sum as f64);
    }

    #[test]
    fn flow_history_default_matches_new_totals() {
        let a = FlowHistory::new();
        let b = FlowHistory::default();
        assert_eq!(a.total_sent, b.total_sent);
        assert_eq!(a.total_recv, b.total_recv);
    }

    #[test]
    fn avg_recv_2s_after_rotate_ignores_prior_slot_when_current_only() {
        let mut h = FlowHistory::new();
        h.add_recv(100);
        h.rotate();
        h.add_recv(50);
        assert_eq!(h.avg_recv_2s(), 150.0);
    }

    #[test]
    fn avg_sent_2s_after_three_rotations_sums_last_two_slots() {
        let mut h = FlowHistory::new();
        h.add_sent(10);
        h.rotate();
        h.add_sent(20);
        h.rotate();
        h.add_sent(100);
        assert_eq!(h.avg_sent_2s(), 120.0);
    }

    #[test]
    fn fifty_rotations_keep_sent_deque_at_most_history_slots() {
        let mut h = FlowHistory::new();
        for _ in 0..50 {
            h.add_sent(1);
            h.rotate();
        }
        assert!(h.sent.len() <= super::HISTORY_SLOTS);
        assert!(h.recv.len() <= super::HISTORY_SLOTS);
    }

    #[test]
    fn recv_peak_zero_when_only_sent_in_slot() {
        let mut h = FlowHistory::new();
        h.add_sent(500);
        h.rotate();
        assert_eq!(h.peak_recv, 0.0);
        assert_eq!(h.peak_sent, 500.0);
    }

    #[test]
    fn sent_peak_zero_when_only_recv_in_slot() {
        let mut h = FlowHistory::new();
        h.add_recv(400);
        h.rotate();
        assert_eq!(h.peak_sent, 0.0);
        assert_eq!(h.peak_recv, 400.0);
    }

    #[test]
    fn avg_recv_40s_sums_up_to_forty_slots() {
        let mut h = FlowHistory::new();
        for i in 0..15 {
            h.add_recv((i + 1) as u64);
            h.rotate();
        }
        let sum: u64 = (1..=15).sum();
        assert_eq!(h.avg_recv_40s(), sum as f64);
    }

    #[test]
    fn avg_sent_10s_with_three_slots_partial() {
        let mut h = FlowHistory::new();
        h.add_sent(1);
        h.rotate();
        h.add_sent(2);
        h.rotate();
        h.add_sent(4);
        assert_eq!(h.avg_sent_10s(), 7.0);
    }

    #[test]
    fn last_seen_updates_on_both_sent_and_recv() {
        let mut h = FlowHistory::new();
        let t0 = h.last_seen;
        std::thread::sleep(std::time::Duration::from_millis(2));
        h.add_sent(1);
        let t1 = h.last_seen;
        std::thread::sleep(std::time::Duration::from_millis(2));
        h.add_recv(1);
        assert!(t1 >= t0);
        assert!(h.last_seen >= t1);
    }

    #[test]
    fn rotate_without_adding_bytes_records_zero_peak_for_empty_second() {
        let mut h = FlowHistory::new();
        h.add_sent(50);
        h.rotate();
        h.rotate();
        assert_eq!(h.peak_sent, 50.0);
    }

    #[test]
    fn cumulative_totals_survive_many_empty_rotates() {
        let mut h = FlowHistory::new();
        h.add_sent(999);
        for _ in 0..20 {
            h.rotate();
        }
        assert_eq!(h.total_sent, 999);
    }

    #[test]
    fn cumulative_recv_totals_survive_many_empty_rotates() {
        let mut h = FlowHistory::new();
        h.add_recv(888);
        for _ in 0..20 {
            h.rotate();
        }
        assert_eq!(h.total_recv, 888);
    }

    #[test]
    fn avg_sent_40s_after_single_slot_is_same_as_2s() {
        let mut h = FlowHistory::new();
        h.add_sent(777);
        assert_eq!(h.avg_sent_40s(), h.avg_sent_2s());
    }

    #[test]
    fn avg_recv_40s_after_single_slot_is_same_as_2s() {
        let mut h = FlowHistory::new();
        h.add_recv(888);
        assert_eq!(h.avg_recv_40s(), h.avg_recv_2s());
    }

    #[test]
    fn recv_deque_same_len_as_sent_after_mixed_rotations() {
        let mut h = FlowHistory::new();
        for _ in 0..12 {
            h.add_sent(1);
            h.add_recv(2);
            h.rotate();
        }
        assert_eq!(h.sent.len(), h.recv.len());
    }

    #[test]
    fn peak_sent_retains_max_rate_across_slots() {
        let mut h = FlowHistory::new();
        h.add_sent(100);
        h.rotate();
        h.add_sent(500);
        h.rotate();
        h.add_sent(200);
        h.rotate();
        assert_eq!(h.peak_sent, 500.0);
    }

    #[test]
    fn peak_recv_retains_max_rate_across_slots() {
        let mut h = FlowHistory::new();
        h.add_recv(10);
        h.rotate();
        h.add_recv(400);
        h.rotate();
        h.add_recv(50);
        h.rotate();
        assert_eq!(h.peak_recv, 400.0);
    }

    #[test]
    fn peak_sent_and_peak_recv_tracked_independently() {
        let mut h = FlowHistory::new();
        h.add_sent(10_000);
        h.rotate();
        h.add_recv(5_000);
        h.rotate();
        assert_eq!(h.peak_sent, 10_000.0);
        assert_eq!(h.peak_recv, 5_000.0);
    }

    #[test]
    fn avg_recv_2s_after_two_rotations_sums_last_two_slots() {
        let mut h = FlowHistory::new();
        h.add_recv(100);
        h.rotate();
        h.add_recv(200);
        h.rotate();
        h.add_recv(50);
        assert_eq!(h.avg_recv_2s(), 250.0);
    }

    #[test]
    fn avg_sent_2s_zero_when_only_recv_in_current_slot() {
        let mut h = FlowHistory::new();
        h.add_recv(999);
        assert_eq!(h.avg_sent_2s(), 0.0);
        assert_eq!(h.avg_recv_2s(), 999.0);
    }

    #[test]
    fn avg_recv_2s_zero_when_only_sent_in_current_slot() {
        let mut h = FlowHistory::new();
        h.add_sent(888);
        assert_eq!(h.avg_recv_2s(), 0.0);
        assert_eq!(h.avg_sent_2s(), 888.0);
    }

    #[test]
    fn add_sent_accumulates_within_current_slot() {
        let mut h = FlowHistory::new();
        h.add_sent(10);
        h.add_sent(20);
        assert_eq!(h.avg_sent_2s(), 30.0);
    }

    #[test]
    fn total_sent_and_recv_independent_in_same_slot() {
        let mut h = FlowHistory::new();
        h.add_sent(11);
        h.add_recv(22);
        assert_eq!(h.total_sent, 11);
        assert_eq!(h.total_recv, 22);
    }

    #[test]
    fn add_recv_zero_leaves_total_recv_zero() {
        let mut h = FlowHistory::new();
        h.add_recv(0);
        assert_eq!(h.total_recv, 0);
    }

    #[test]
    fn add_sent_zero_leaves_total_sent_zero() {
        let mut h = FlowHistory::new();
        h.add_sent(0);
        assert_eq!(h.total_sent, 0);
    }

    #[test]
    fn avg_sent_2s_sums_completed_and_current_slot_after_rotate() {
        let mut h = FlowHistory::new();
        h.add_sent(100);
        h.rotate();
        h.add_sent(50);
        assert_eq!(h.avg_sent_2s(), 150.0);
    }

    #[test]
    fn avg_recv_2s_sums_completed_and_current_slot_after_rotate() {
        let mut h = FlowHistory::new();
        h.add_recv(200);
        h.rotate();
        h.add_recv(50);
        assert_eq!(h.avg_recv_2s(), 250.0);
    }

    #[test]
    fn avg_sent_10s_sums_all_slots_when_fewer_than_ten_exist() {
        let mut h = FlowHistory::new();
        for i in 1..=9 {
            h.add_sent(i);
            h.rotate();
        }
        // Deque has 10 slots ending with 0 in the newest; window takes all 10 → 1+2+…+9+0 = 45.
        assert_eq!(h.avg_sent_10s(), 45.0);
    }

    #[test]
    fn avg_recv_10s_sums_all_slots_when_fewer_than_ten_exist() {
        let mut h = FlowHistory::new();
        for i in 1..=9 {
            h.add_recv(i);
            h.rotate();
        }
        assert_eq!(h.avg_recv_10s(), 45.0);
    }

    #[test]
    fn avg_recv_10s_with_three_slots_partial() {
        let mut h = FlowHistory::new();
        h.add_recv(1);
        h.rotate();
        h.add_recv(2);
        h.rotate();
        h.add_recv(4);
        assert_eq!(h.avg_recv_10s(), 7.0);
    }

    #[test]
    fn avg_sent_40s_sums_eleven_slots_when_only_eleven_exist() {
        let mut h = FlowHistory::new();
        for i in 1..=11 {
            h.add_sent(i);
            h.rotate();
        }
        // 12 slots ending with 0 in newest; window takes all → 1+2+…+11+0 = 66.
        assert_eq!(h.avg_sent_40s(), 66.0);
    }

    #[test]
    fn avg_recv_40s_sums_eleven_slots_when_only_eleven_exist() {
        let mut h = FlowHistory::new();
        for i in 1..=11 {
            h.add_recv(i);
            h.rotate();
        }
        assert_eq!(h.avg_recv_40s(), 66.0);
    }
}
