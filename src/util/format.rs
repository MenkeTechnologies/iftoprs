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
