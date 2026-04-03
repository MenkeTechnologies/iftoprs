use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex, OnceLock};

use crate::data::flow::Protocol;

/// Cached socket→process table. Refreshed periodically by the proc-lookup thread.
/// Uses `lsof -i -n -P -F pcn` to get ALL socket→pid mappings at once,
/// rather than per-flow lookups.
static PROC_CACHE: OnceLock<Arc<Mutex<ProcTable>>> = OnceLock::new();

struct ProcTable {
    /// Maps (local_port, protocol) → (pid, name)
    by_port: HashMap<(u16, u8), (u32, String)>,
}

fn get_cache() -> &'static Arc<Mutex<ProcTable>> {
    PROC_CACHE.get_or_init(|| {
        Arc::new(Mutex::new(ProcTable {
            by_port: HashMap::new(),
        }))
    })
}

/// Refresh the entire process→socket table. Call this periodically from the
/// proc-lookup thread (e.g. every 2s). Much more efficient than per-flow lsof.
pub fn refresh_proc_table() {
    #[cfg(target_os = "macos")]
    {
        refresh_proc_table_lsof();
    }
    #[cfg(target_os = "linux")]
    {
        refresh_proc_table_linux();
    }
}

#[cfg(target_os = "macos")]
fn refresh_proc_table_lsof() {
    use std::process::Command;

    // lsof -i -n -P -F pcn  — list ALL network sockets with pid, command, name
    let output = match Command::new("lsof")
        .args(["-i", "-n", "-P", "-F", "pcn"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return,
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut new_table: HashMap<(u16, u8), (u32, String)> = HashMap::new();
    let mut current_pid: Option<u32> = None;
    let mut current_name: Option<String> = None;

    for line in stdout.lines() {
        if let Some(p) = line.strip_prefix('p') {
            current_pid = p.parse().ok();
            current_name = None;
        } else if let Some(n) = line.strip_prefix('c') {
            current_name = Some(n.to_string());
        } else if let Some(n) = line.strip_prefix('n') {
            // n field: "host:port->remote:port" or "*:port" etc
            if let (Some(pid), Some(name)) = (current_pid, &current_name) {
                // Extract local port from patterns like:
                //   *:443 or 127.0.0.1:8080 or [::1]:443 or host:port->remote:port
                if let Some(local_port) = extract_local_port(n) {
                    // Determine protocol from the lsof line (TCP vs UDP)
                    // lsof -F doesn't give protocol directly in 'n' field,
                    // so we insert for both TCP and UDP
                    new_table
                        .entry((local_port, 6))
                        .or_insert((pid, name.clone()));
                    new_table
                        .entry((local_port, 17))
                        .or_insert((pid, name.clone()));
                }
            }
        }
    }

    let cache = get_cache();
    let mut table = cache.lock().unwrap_or_else(|e| e.into_inner());
    table.by_port = new_table;
}

#[cfg(target_os = "macos")]
fn extract_local_port(n_field: &str) -> Option<u16> {
    // Strip the connection part (->remote) if present
    let local = n_field.split("->").next()?;
    // Find the last colon — port is after it
    let colon_pos = local.rfind(':')?;
    let port_str = &local[colon_pos + 1..];
    port_str.parse().ok()
}

#[cfg(target_os = "linux")]
fn refresh_proc_table_linux() {
    // Parse /proc/net/tcp and /proc/net/tcp6 for inode→port mapping,
    // then walk /proc/[pid]/fd/ to map inode→pid
    use std::fs;

    let mut inode_to_port: HashMap<u64, (u16, u8)> = HashMap::new();

    for (path, proto) in [
        ("/proc/net/tcp", 6u8),
        ("/proc/net/tcp6", 6),
        ("/proc/net/udp", 17),
        ("/proc/net/udp6", 17),
    ] {
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines().skip(1) {
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() < 10 {
                    continue;
                }
                // fields[1] = local_address:port (hex)
                if let Some(port) = parse_proc_net_port(fields[1])
                    && let Ok(inode) = fields[9].parse::<u64>()
                    && inode > 0
                {
                    inode_to_port.insert(inode, (port, proto));
                }
            }
        }
    }

    let mut new_table: HashMap<(u16, u8), (u32, String)> = HashMap::new();

    if let Ok(proc_entries) = fs::read_dir("/proc") {
        for entry in proc_entries.flatten() {
            let pid_str = entry.file_name();
            let pid_str = pid_str.to_string_lossy();
            let pid: u32 = match pid_str.parse() {
                Ok(p) => p,
                Err(_) => continue,
            };

            let fd_path = format!("/proc/{}/fd", pid);
            let comm_path = format!("/proc/{}/comm", pid);
            let name = fs::read_to_string(&comm_path)
                .unwrap_or_else(|_| format!("pid:{}", pid))
                .trim()
                .to_string();

            if let Ok(fds) = fs::read_dir(&fd_path) {
                for fd in fds.flatten() {
                    if let Ok(link) = fs::read_link(fd.path()) {
                        let link_str = link.to_string_lossy();
                        if let Some(inode_str) = link_str
                            .strip_prefix("socket:[")
                            .and_then(|s| s.strip_suffix(']'))
                            && let Ok(inode) = inode_str.parse::<u64>()
                            && let Some(&(port, proto)) = inode_to_port.get(&inode)
                        {
                            new_table
                                .entry((port, proto))
                                .or_insert((pid, name.clone()));
                        }
                    }
                }
            }
        }
    }

    let cache = get_cache();
    let mut table = cache.lock().unwrap_or_else(|e| e.into_inner());
    table.by_port = new_table;
}

#[cfg(target_os = "linux")]
fn parse_proc_net_port(addr_port: &str) -> Option<u16> {
    let colon = addr_port.rfind(':')?;
    u16::from_str_radix(&addr_port[colon + 1..], 16).ok()
}

/// Look up the owning process for a network flow.
pub fn lookup_process(
    _src: IpAddr,
    src_port: u16,
    _dst: IpAddr,
    dst_port: u16,
    protocol: &Protocol,
) -> Option<(u32, String)> {
    let proto_num: u8 = match protocol {
        Protocol::Tcp => 6,
        Protocol::Udp => 17,
        _ => return None,
    };

    let cache = get_cache();
    let table = cache.lock().unwrap_or_else(|e| e.into_inner());

    // Try local port (src_port first, then dst_port)
    if let Some(entry) = table.by_port.get(&(src_port, proto_num)) {
        return Some(entry.clone());
    }
    if let Some(entry) = table.by_port.get(&(dst_port, proto_num)) {
        return Some(entry.clone());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_cache_returns_same_instance() {
        let a = get_cache() as *const _;
        let b = get_cache() as *const _;
        assert_eq!(a, b);
    }

    #[test]
    fn lookup_process_unknown_port() {
        let result = lookup_process(
            "10.0.0.1".parse().unwrap(),
            60000,
            "10.0.0.2".parse().unwrap(),
            60001,
            &Protocol::Tcp,
        );
        // May or may not find something depending on system state, but should not panic
        let _ = result;
    }

    #[test]
    fn lookup_process_unsupported_protocol() {
        let result = lookup_process(
            "10.0.0.1".parse().unwrap(),
            80,
            "10.0.0.2".parse().unwrap(),
            80,
            &Protocol::Other(47),
        );
        assert!(result.is_none());
    }

    #[test]
    fn lookup_process_icmp_returns_none() {
        let result = lookup_process(
            "10.0.0.1".parse().unwrap(),
            0,
            "10.0.0.2".parse().unwrap(),
            0,
            &Protocol::Icmp,
        );
        assert!(result.is_none());
    }

    #[test]
    fn lookup_process_udp_no_panic() {
        let result = lookup_process(
            "10.0.0.1".parse().unwrap(),
            53,
            "8.8.8.8".parse().unwrap(),
            53,
            &Protocol::Udp,
        );
        let _ = result;
    }

    #[test]
    fn refresh_proc_table_no_panic() {
        // Should not panic regardless of system state
        refresh_proc_table();
    }

    #[test]
    fn refresh_proc_table_twice_no_panic() {
        refresh_proc_table();
        refresh_proc_table();
    }

    #[test]
    fn mutex_poison_recovery() {
        let cache = get_cache();
        let cache_clone = Arc::clone(cache);
        let h = std::thread::spawn(move || {
            let _guard = cache_clone.lock().unwrap();
            panic!("intentional poison");
        });
        let _ = h.join();

        // lookup_process should recover from poisoned mutex
        let result = lookup_process(
            "10.0.0.1".parse().unwrap(),
            80,
            "10.0.0.2".parse().unwrap(),
            80,
            &Protocol::Tcp,
        );
        let _ = result; // should not panic
    }

    #[cfg(target_os = "macos")]
    mod macos_tests {
        use super::super::*;

        #[test]
        fn extract_local_port_simple() {
            assert_eq!(extract_local_port("*:443"), Some(443));
        }

        #[test]
        fn extract_local_port_ipv4() {
            assert_eq!(extract_local_port("127.0.0.1:8080"), Some(8080));
        }

        #[test]
        fn extract_local_port_with_remote() {
            assert_eq!(extract_local_port("10.0.0.1:5000->10.0.0.2:80"), Some(5000));
        }

        #[test]
        fn extract_local_port_ipv6() {
            assert_eq!(extract_local_port("[::1]:443"), Some(443));
        }

        #[test]
        fn extract_local_port_invalid() {
            assert_eq!(extract_local_port("no-colon"), None);
        }

        #[test]
        fn extract_local_port_non_numeric() {
            assert_eq!(extract_local_port("host:abc"), None);
        }

        #[test]
        fn extract_local_port_wildcard() {
            assert_eq!(extract_local_port("*:22"), Some(22));
        }

        #[test]
        fn refresh_proc_table_lsof_populates() {
            refresh_proc_table_lsof();
            // After refresh, cache should exist (may be empty if no sockets)
            let cache = get_cache();
            let table = cache.lock().unwrap_or_else(|e| e.into_inner());
            let _ = table.by_port.len(); // just verify access
        }
    }

    #[cfg(target_os = "linux")]
    mod linux_tests {
        use super::super::parse_proc_net_port;

        #[test]
        fn parse_proc_net_port_ipv4_hex() {
            assert_eq!(parse_proc_net_port("0100007F:0016"), Some(22));
        }

        #[test]
        fn parse_proc_net_port_ipv6_hex() {
            assert_eq!(
                parse_proc_net_port("00000000000000000000000000000000:0050"),
                Some(80)
            );
        }

        #[test]
        fn parse_proc_net_port_invalid_hex() {
            assert_eq!(parse_proc_net_port("0100007F:GGGG"), None);
        }

        #[test]
        fn parse_proc_net_port_no_colon() {
            assert_eq!(parse_proc_net_port("nope"), None);
        }
    }
}
