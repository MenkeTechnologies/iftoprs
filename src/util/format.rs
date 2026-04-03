/// Format a byte rate into a human-readable string.
/// If `use_bytes` is false, converts to bits and uses iftop-style units (kb, Mb, Gb).
/// If `use_bytes` is true, uses KB, MB, GB.
pub fn readable_size(bytes_per_sec: f64, use_bytes: bool) -> String {
    let (value, units) = if use_bytes {
        (bytes_per_sec, ["B", "KB", "MB", "GB", "TB"])
    } else {
        (bytes_per_sec * 8.0, ["b", "kb", "Mb", "Gb", "Tb"])
    };

    if value < 1_000.0 {
        format!("{:.0}{}", value, units[0])
    } else if value < 1_000_000.0 {
        format!("{:.2}{}", value / 1_000.0, units[1])
    } else if value < 1_000_000_000.0 {
        format!("{:.2}{}", value / 1_000_000.0, units[2])
    } else if value < 1_000_000_000_000.0 {
        format!("{:.2}{}", value / 1_000_000_000.0, units[3])
    } else {
        format!("{:.2}{}", value / 1_000_000_000_000.0, units[4])
    }
}

/// Format cumulative byte count.
pub fn readable_total(bytes: u64, use_bytes: bool) -> String {
    // Cumulative totals always display in bytes regardless of mode
    let value = bytes as f64;
    let units = ["B", "KB", "MB", "GB", "TB"];
    let _ = use_bytes;

    if value < 1_000.0 {
        format!("{:.0}{}", value, units[0])
    } else if value < 1_000_000.0 {
        format!("{:.1}{}", value / 1_000.0, units[1])
    } else if value < 1_000_000_000.0 {
        format!("{:.0}{}", value / 1_000_000.0, units[2])
    } else if value < 1_000_000_000_000.0 {
        format!("{:.2}{}", value / 1_000_000_000.0, units[3])
    } else {
        format!("{:.2}{}", value / 1_000_000_000_000.0, units[4])
    }
}

/// Render a sparkline string from a slice of values.
/// Uses Unicode block elements: ▁▂▃▄▅▆▇█
pub fn sparkline(data: &[u64], max_width: usize) -> String {
    const BLOCKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    if data.is_empty() {
        return String::new();
    }

    // Take the last `max_width` values
    let start = data.len().saturating_sub(max_width);
    let slice = &data[start..];

    let max = slice.iter().copied().max().unwrap_or(1).max(1);

    slice
        .iter()
        .map(|&v| {
            if v == 0 {
                ' '
            } else {
                let idx = ((v as f64 / max as f64) * 7.0).round() as usize;
                BLOCKS[idx.min(7)]
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readable_size_bits_zero() {
        assert_eq!(readable_size(0.0, false), "0b");
    }

    #[test]
    fn readable_size_bits_small() {
        assert_eq!(readable_size(100.0, false), "800b");
    }

    #[test]
    fn readable_size_bits_kilobits() {
        assert_eq!(readable_size(1000.0, false), "8.00kb");
    }

    #[test]
    fn readable_size_bits_megabits() {
        assert_eq!(readable_size(1_000_000.0, false), "8.00Mb");
    }

    #[test]
    fn readable_size_bits_gigabits() {
        assert_eq!(readable_size(125_000_000.0, false), "1.00Gb");
    }

    #[test]
    fn readable_size_bytes_mode() {
        assert_eq!(readable_size(0.0, true), "0B");
        assert_eq!(readable_size(500.0, true), "500B");
        assert_eq!(readable_size(1500.0, true), "1.50KB");
        assert_eq!(readable_size(1_500_000.0, true), "1.50MB");
    }

    #[test]
    fn readable_total_small() {
        assert_eq!(readable_total(0, false), "0B");
        assert_eq!(readable_total(999, false), "999B");
    }

    #[test]
    fn readable_total_kilobytes() {
        assert_eq!(readable_total(1500, false), "1.5KB");
    }

    #[test]
    fn readable_total_megabytes() {
        assert_eq!(readable_total(5_000_000, false), "5MB");
    }

    #[test]
    fn readable_total_gigabytes() {
        assert_eq!(readable_total(1_500_000_000, false), "1.50GB");
    }

    #[test]
    fn readable_total_terabytes() {
        assert_eq!(readable_total(2_000_000_000_000, false), "2.00TB");
    }

    // ── Additional readable_size tests ──

    #[test]
    fn readable_size_bits_terabits() {
        assert_eq!(readable_size(125_000_000_000.0, false), "1.00Tb");
    }

    #[test]
    fn readable_size_bytes_gigabytes() {
        assert_eq!(readable_size(1_500_000_000.0, true), "1.50GB");
    }

    #[test]
    fn readable_size_bytes_terabytes() {
        assert_eq!(readable_size(2_000_000_000_000.0, true), "2.00TB");
    }

    #[test]
    fn readable_size_bytes_zero() {
        assert_eq!(readable_size(0.0, true), "0B");
    }

    #[test]
    fn readable_size_boundary_999() {
        let r = readable_size(999.0, true);
        assert!(r.contains("B") && !r.contains("K"));
    }

    #[test]
    fn readable_size_boundary_1000() {
        let r = readable_size(1000.0, true);
        assert!(r.contains("KB"));
    }

    #[test]
    fn readable_size_boundary_999999() {
        let r = readable_size(999_999.0, true);
        assert!(r.contains("KB"));
    }

    #[test]
    fn readable_size_boundary_1000000() {
        let r = readable_size(1_000_000.0, true);
        assert!(r.contains("MB"));
    }

    #[test]
    fn readable_size_fractional_bytes() {
        let r = readable_size(0.5, true);
        assert_eq!(r, "0B"); // rounds to 0
    }

    #[test]
    fn readable_size_bits_exact_boundary() {
        // 125 bytes/s = 1000 bits/s = 1.00kb
        assert_eq!(readable_size(125.0, false), "1.00kb");
    }

    // ── Additional readable_total tests ──

    #[test]
    fn readable_total_exactly_1000() {
        assert_eq!(readable_total(1000, false), "1.0KB");
    }

    #[test]
    fn readable_total_use_bytes_flag_ignored() {
        // readable_total always uses bytes regardless of flag
        assert_eq!(readable_total(500, true), readable_total(500, false));
    }

    #[test]
    fn readable_total_large_value() {
        let r = readable_total(10_000_000_000_000, false);
        assert!(r.contains("TB"));
    }

    #[test]
    fn readable_total_u64_max() {
        let r = readable_total(u64::MAX, false);
        assert!(!r.is_empty());
    }

    // ── Sparkline ──

    #[test]
    fn sparkline_empty() {
        assert_eq!(sparkline(&[], 10), "");
    }

    #[test]
    fn sparkline_single_value() {
        let s = sparkline(&[100], 10);
        assert_eq!(s.chars().count(), 1);
        assert_eq!(s, "█");
    }

    #[test]
    fn sparkline_all_zeros() {
        let s = sparkline(&[0, 0, 0, 0], 10);
        assert_eq!(s, "    ");
    }

    #[test]
    fn sparkline_ascending() {
        let s = sparkline(&[0, 25, 50, 75, 100], 10);
        assert_eq!(s.chars().count(), 5);
        // Last char should be the tallest block
        assert_eq!(s.chars().last().unwrap(), '█');
    }

    #[test]
    fn sparkline_truncates_to_max_width() {
        let data: Vec<u64> = (0..100).collect();
        let s = sparkline(&data, 20);
        assert_eq!(s.chars().count(), 20);
    }

    #[test]
    fn sparkline_uses_recent_data() {
        let data: Vec<u64> = (0..100).collect();
        let s = sparkline(&data, 5);
        // Should use the last 5 values (95..100), all high
        assert_eq!(s.chars().count(), 5);
    }

    #[test]
    fn sparkline_uniform_values() {
        let s = sparkline(&[50, 50, 50, 50], 10);
        // All same value → all should be the same block (max block since v/max = 1.0)
        let chars: Vec<char> = s.chars().collect();
        assert!(chars.iter().all(|&c| c == chars[0]));
    }

    #[test]
    fn sparkline_one_spike() {
        let s = sparkline(&[0, 0, 100, 0, 0], 10);
        let chars: Vec<char> = s.chars().collect();
        assert_eq!(chars[2], '█');
        assert_eq!(chars[0], ' ');
        assert_eq!(chars[4], ' ');
    }

    #[test]
    fn sparkline_max_width_zero_returns_empty() {
        let s = sparkline(&[1, 2, 3, 4], 0);
        assert_eq!(s, "");
    }

    #[test]
    fn sparkline_single_nonzero_max() {
        let s = sparkline(&[u64::MAX], 5);
        assert_eq!(s, "█");
    }

    #[test]
    fn sparkline_very_large_values() {
        let s = sparkline(&[1_000_000, 2_000_000, 3_000_000], 3);
        assert_eq!(s.chars().count(), 3);
        assert_eq!(s.chars().last().unwrap(), '█');
    }

    #[test]
    fn readable_total_exactly_one_million_bytes() {
        assert_eq!(readable_total(1_000_000, true), "1MB");
    }

    #[test]
    fn readable_total_one_byte() {
        assert_eq!(readable_total(1, false), "1B");
    }

    #[test]
    fn sparkline_two_values_min_max() {
        // max=16 so 1 maps to block 0 (▁) and 16 maps to block 7 (█)
        let s = sparkline(&[1, 16], 10);
        assert_eq!(s.chars().count(), 2);
        let chars: Vec<char> = s.chars().collect();
        assert_eq!(chars[0], '▁');
        assert_eq!(chars[1], '█');
    }

    #[test]
    fn readable_size_bytes_just_under_kilobyte() {
        assert_eq!(readable_size(999.0, true), "999B");
    }

    #[test]
    fn readable_size_bits_one_megabit_exact() {
        assert_eq!(readable_size(125_000.0, false), "1.00Mb");
    }

    #[test]
    fn readable_total_boundary_kb() {
        let r = readable_total(999_999, false);
        assert!(r.contains("KB") || r.contains("MB"));
    }

    #[test]
    fn sparkline_max_width_exceeds_len() {
        let s = sparkline(&[1, 2, 3], 100);
        assert_eq!(s.chars().count(), 3);
    }

    #[test]
    fn sparkline_all_max_except_one_zero() {
        let s = sparkline(&[0, 100, 100, 100], 10);
        let chars: Vec<char> = s.chars().collect();
        assert_eq!(chars[0], ' ');
    }

    #[test]
    fn readable_size_bytes_terabit_tier() {
        let r = readable_size(3_000_000_000_000.0, true);
        assert!(r.contains("TB"));
    }

    #[test]
    fn readable_size_bits_gigabit_tier() {
        let r = readable_size(125_000_000.0, false);
        assert!(r.contains("Gb"));
    }

    #[test]
    fn readable_total_zero_bytes_explicit() {
        assert_eq!(readable_total(0, true), "0B");
    }

    #[test]
    fn sparkline_length_one_max_width_one() {
        assert_eq!(sparkline(&[42], 1).chars().count(), 1);
    }

    #[test]
    fn readable_size_bits_exactly_one_kb() {
        assert_eq!(readable_size(125.0, false), "1.00kb");
    }

    #[test]
    fn readable_total_mb_no_fraction_when_round() {
        let r = readable_total(2_000_000, false);
        assert!(r.contains("MB"));
    }

    #[test]
    fn sparkline_max_width_one_takes_last_sample() {
        let data: Vec<u64> = (0..50).collect();
        let s = sparkline(&data, 1);
        assert_eq!(s.chars().count(), 1);
    }

    #[test]
    fn readable_total_gigabyte_range() {
        let r = readable_total(3_500_000_000, false);
        assert!(r.contains("GB"));
    }

    #[test]
    fn readable_total_exactly_one_gigabyte() {
        assert_eq!(readable_total(1_000_000_000, false), "1.00GB");
    }

    #[test]
    fn readable_size_bytes_megabyte_exact_from_table() {
        assert_eq!(readable_size(1_500_000.0, true), "1.50MB");
    }

    #[test]
    fn sparkline_three_plateau() {
        let s = sparkline(&[10, 10, 10], 10);
        let c: Vec<char> = s.chars().collect();
        assert!(c.iter().all(|&x| x == c[0]));
    }

    #[test]
    fn readable_size_bits_terabit_tier() {
        let r = readable_size(125_000_000_000.0, false);
        assert!(r.contains("Tb"));
    }

    #[test]
    fn readable_total_terabyte_range() {
        let r = readable_total(5_000_000_000_000, false);
        assert!(r.contains("TB"));
    }

    #[test]
    fn readable_size_bytes_exactly_1000_boundary() {
        assert_eq!(readable_size(1000.0, true), "1.00KB");
    }

    #[test]
    fn sparkline_two_nonzero_different_heights() {
        let s = sparkline(&[1, 8], 10);
        assert_eq!(s.chars().count(), 2);
        let mut chars = s.chars();
        assert_ne!(chars.next(), chars.next());
    }

    #[test]
    fn sparkline_three_values_spans_blocks() {
        let s = sparkline(&[1, 50, 100], 10);
        assert_eq!(s.chars().count(), 3);
        let v: Vec<char> = s.chars().collect();
        assert_ne!(v[0], v[2]);
    }

    #[test]
    fn readable_size_bits_sub_one_kb_stays_b() {
        assert_eq!(readable_size(124.0, false), "992b");
    }

    #[test]
    fn readable_size_bytes_sub_one_kb_stays_b() {
        assert_eq!(readable_size(999.4, true), "999B");
    }

    #[test]
    fn sparkline_singleton_nonzero_is_tallest_block() {
        let s = sparkline(&[42], 5);
        assert_eq!(s, "█");
    }

    #[test]
    fn readable_size_megabytes_mode_two_decimals() {
        assert_eq!(readable_size(2_500_000.0, true), "2.50MB");
    }

    #[test]
    fn sparkline_four_steps_monotonic_increasing() {
        let s = sparkline(&[1, 2, 3, 4], 10);
        let c: Vec<char> = s.chars().collect();
        assert!(c[0] < c[1]);
        assert!(c[1] < c[3]);
    }

    #[test]
    fn readable_total_just_below_one_million_bytes_kb() {
        assert_eq!(readable_total(999_999, false), "1000.0KB");
    }

    #[test]
    fn readable_size_gigabytes_bytes_mode() {
        assert_eq!(readable_size(3_500_000_000.0, true), "3.50GB");
    }

    #[test]
    fn sparkline_all_same_nonzero_uniform_blocks() {
        let s = sparkline(&[7, 7, 7, 7], 10);
        let c: Vec<char> = s.chars().collect();
        assert!(c.iter().all(|&x| x == c[0]));
    }

    #[test]
    fn sparkline_strictly_descending_heights() {
        let s = sparkline(&[100, 75, 50, 25, 1], 10);
        let c: Vec<char> = s.chars().collect();
        assert!(c[0] > c[1]);
        assert!(c[1] > c[2]);
        assert!(c[3] > c[4]);
    }

    #[test]
    fn readable_size_bits_two_gigabit() {
        assert_eq!(readable_size(250_000_000.0, false), "2.00Gb");
    }

    #[test]
    fn readable_total_exactly_one_terabyte() {
        assert_eq!(readable_total(1_000_000_000_000, false), "1.00TB");
    }

    #[test]
    fn readable_size_zero_point_one_bytes_shows_zero() {
        assert_eq!(readable_size(0.1, true), "0B");
    }

    #[test]
    fn readable_total_just_under_one_gb_rounds_to_1000_mb() {
        // 999_999_999 B is just under 1 GB; MB branch uses {:.0} on the megabyte value.
        assert_eq!(readable_total(999_999_999, false), "1000MB");
    }

    #[test]
    fn readable_total_just_under_one_tb_rounds_to_1000_tb() {
        // Last TB branch: float rounding can show 1000.00 TB at the u64 edge.
        assert_eq!(readable_total(999_999_999_999_999, false), "1000.00TB");
    }

    #[test]
    fn readable_total_just_under_one_tb_in_gb_branch_rounds_to_1000_gb() {
        assert_eq!(readable_total(999_999_999_999, false), "1000.00GB");
    }

    #[test]
    fn readable_size_bytes_one_tenth_below_kb_boundary() {
        assert_eq!(readable_size(999.4, true), "999B");
    }

    #[test]
    fn sparkline_two_identical_nonzero_plateau() {
        let s = sparkline(&[42, 42, 42], 10);
        let c: Vec<char> = s.chars().collect();
        assert!(c.iter().all(|&x| x == c[0]));
    }

    #[test]
    fn readable_size_bits_just_under_one_megabit_per_second() {
        // 999_999 B/s → 7_999_992 b/s → megabit tier (not kilobit).
        assert_eq!(readable_size(999_999.0, false), "8.00Mb");
    }

    #[test]
    fn readable_size_bytes_exactly_one_million_per_second() {
        assert_eq!(readable_size(1_000_000.0, true), "1.00MB");
    }

    #[test]
    fn readable_size_bits_last_value_before_kb_tier_rounds_to_four_digits() {
        // 124.999 B/s → 999.992 b/s, still in the sub-kilobit display bucket (< 1000 b label).
        assert_eq!(readable_size(124.999, false), "1000b");
    }

    #[test]
    fn readable_size_bits_exactly_one_kilobit_per_second_from_bytes() {
        // 125 B/s → 1000 b/s → kilobit tier.
        assert_eq!(readable_size(125.0, false), "1.00kb");
    }

    #[test]
    fn readable_size_bytes_two_terabytes_per_second() {
        assert_eq!(readable_size(2_000_000_000_000.0, true), "2.00TB");
    }

    #[test]
    fn readable_total_exactly_ten_kilobytes() {
        assert_eq!(readable_total(10_000, false), "10.0KB");
    }

    #[test]
    fn sparkline_min_max_span_uses_full_block_range() {
        let s = sparkline(&[0, 1, 2, 3, 4, 5, 6, 7], 20);
        let blocks: Vec<char> = s.chars().collect();
        assert!(blocks.iter().any(|&c| c == '▁' || c == ' '));
        assert!(blocks.contains(&'█'));
    }

    #[test]
    fn readable_size_bytes_five_hundred_kilobytes_per_second() {
        assert_eq!(readable_size(500_000.0, true), "500.00KB");
    }

    #[test]
    fn readable_size_bits_one_gigabit_per_second() {
        assert_eq!(readable_size(125_000_000.0, false), "1.00Gb");
    }

    #[test]
    fn readable_size_bits_half_byte_per_second() {
        assert_eq!(readable_size(0.5, false), "4b");
    }
}
