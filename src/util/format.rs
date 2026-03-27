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
    let (value, units) = if use_bytes {
        (bytes as f64, ["B", "KB", "MB", "GB", "TB"])
    } else {
        (bytes as f64, ["B", "KB", "MB", "GB", "TB"]) // cumulative always in bytes
    };

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
}
