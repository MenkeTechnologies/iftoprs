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
        let r = readable_total(1000, false);
        assert!(r.contains("KB"));
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
}
