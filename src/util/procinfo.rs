use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use crate::data::flow::Protocol;

/// Cached socket→process table. Refreshed periodically by the proc-lookup thread.
/// Built from native OS facilities — `libproc` syscalls on macOS, `/proc` on
/// Linux — with no external binaries shelled out at runtime.
static PROC_CACHE: OnceLock<Arc<Mutex<ProcTable>>> = OnceLock::new();

struct ProcTable {
    /// Maps (local_port, protocol) → (pid, name)
    by_port: HashMap<(u16, u8), (u32, String)>,
    /// Maps pid → on-disk executable path, for socket-owning processes. Used by
    /// the provenance layer (Publishers view) to fingerprint the binary.
    exe_by_pid: HashMap<u32, PathBuf>,
}

fn get_cache() -> &'static Arc<Mutex<ProcTable>> {
    PROC_CACHE.get_or_init(|| {
        Arc::new(Mutex::new(ProcTable {
            by_port: HashMap::new(),
            exe_by_pid: HashMap::new(),
        }))
    })
}

/// Return the on-disk executable path for a socket-owning PID, if the last
/// table refresh captured it. Feeds `provenance::identity_for`.
pub fn exe_path_for(pid: u32) -> Option<PathBuf> {
    let cache = get_cache();
    let table = cache.lock().unwrap_or_else(|e| e.into_inner());
    table.exe_by_pid.get(&pid).cloned()
}

/// Refresh the entire process→socket table. Call this periodically from the
/// proc-lookup thread (e.g. every 2s). One full enumeration per call, cheaper
/// than a per-flow lookup on every packet.
pub fn refresh_proc_table() {
    #[cfg(target_os = "macos")]
    {
        refresh_proc_table_macos();
    }
    #[cfg(target_os = "linux")]
    {
        refresh_proc_table_linux();
    }
}

/// Convert a network-byte-order port (as stored in `insi_lport`, a `c_int`
/// holding the 16-bit value) to host byte order.
#[cfg(target_os = "macos")]
fn ntohs(net_order: i32) -> u16 {
    u16::from_be(net_order as u16)
}

#[cfg(target_os = "macos")]
fn refresh_proc_table_macos() {
    use libproc::libproc::bsd_info::BSDInfo;
    use libproc::libproc::file_info::{ListFDs, ProcFDType, pidfdinfo};
    use libproc::libproc::net_info::{SocketFDInfo, SocketInfoKind};
    use libproc::libproc::proc_pid::{listpidinfo, name, pidinfo, pidpath};
    use libproc::processes::{ProcFilter, pids_by_type};

    // Enumerate every PID, then every socket fd of each, reading the local
    // port + protocol straight out of the kernel via libproc. No subprocess.
    let pids = match pids_by_type(ProcFilter::All) {
        Ok(p) => p,
        Err(_) => return,
    };

    let mut new_table: HashMap<(u16, u8), (u32, String)> = HashMap::new();
    let mut new_exe: HashMap<u32, PathBuf> = HashMap::new();

    for pid in pids {
        if pid == 0 {
            continue;
        }
        let pid_i = pid as i32;

        // pbi_nfiles sizes the fd buffer. A failure here means the process is
        // gone or we lack the privilege to inspect it — skip it, don't abort.
        let nfiles = match pidinfo::<BSDInfo>(pid_i, 0) {
            Ok(info) => info.pbi_nfiles as usize,
            Err(_) => continue,
        };
        let fds = match listpidinfo::<ListFDs>(pid_i, nfiles) {
            Ok(fds) => fds,
            Err(_) => continue,
        };

        // Resolve the process name lazily — only once we know it owns a socket.
        let mut proc_name: Option<String> = None;

        for fd in fds {
            if !matches!(fd.proc_fdtype.into(), ProcFDType::Socket) {
                continue;
            }
            let sock = match pidfdinfo::<SocketFDInfo>(pid_i, fd.proc_fd) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // soi_proto is a union; the active arm is selected by soi_kind.
            let (port_net, proto): (i32, u8) = match sock.psi.soi_kind.into() {
                SocketInfoKind::Tcp => {
                    let ini = unsafe { sock.psi.soi_proto.pri_tcp.tcpsi_ini };
                    (ini.insi_lport, 6)
                }
                SocketInfoKind::In => {
                    // IPv4/IPv6 datagram socket (UDP and friends).
                    let ini = unsafe { sock.psi.soi_proto.pri_in };
                    (ini.insi_lport, sock.psi.soi_protocol as u8)
                }
                _ => continue,
            };

            // lookup_process only resolves TCP/UDP flows.
            if proto != 6 && proto != 17 {
                continue;
            }
            let port = ntohs(port_net);
            if port == 0 {
                continue;
            }

            // Resolve name and executable path once, the first time this PID is
            // seen to own a socket.
            if proc_name.is_none() {
                proc_name = Some(name(pid_i).unwrap_or_else(|_| format!("pid:{}", pid)));
                if let Ok(path) = pidpath(pid_i) {
                    new_exe.insert(pid, PathBuf::from(path));
                }
            }
            let nm = proc_name.as_ref().expect("resolved above");
            new_table.entry((port, proto)).or_insert((pid, nm.clone()));
        }
    }

    let cache = get_cache();
    let mut table = cache.lock().unwrap_or_else(|e| e.into_inner());
    table.by_port = new_table;
    table.exe_by_pid = new_exe;
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

    // Capture the executable path for every socket-owning PID (for provenance).
    let mut new_exe: HashMap<u32, PathBuf> = HashMap::new();
    let mut pids_seen: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for (pid, _) in new_table.values() {
        if pids_seen.insert(*pid)
            && let Ok(target) = fs::read_link(format!("/proc/{}/exe", pid))
        {
            new_exe.insert(*pid, target);
        }
    }

    let cache = get_cache();
    let mut table = cache.lock().unwrap_or_else(|e| e.into_inner());
    table.by_port = new_table;
    table.exe_by_pid = new_exe;
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

        // insi_lport stores the port in network (big-endian) byte order inside a
        // c_int; ntohs() must return it in host order. 0x5000 -> 80, 0xBB01 -> 443.
        #[test]
        fn ntohs_port_80() {
            assert_eq!(ntohs(0x5000), 80);
        }

        #[test]
        fn ntohs_port_443() {
            assert_eq!(ntohs(0xBB01), 443);
        }

        #[test]
        fn ntohs_port_22() {
            assert_eq!(ntohs(0x1600), 22);
        }

        #[test]
        fn ntohs_port_max() {
            assert_eq!(ntohs(0xFFFF), 65535);
        }

        #[test]
        fn ntohs_port_zero() {
            assert_eq!(ntohs(0x0000), 0);
        }

        #[test]
        fn ntohs_ignores_high_bits() {
            // insi_lport is a c_int; only the low 16 bits carry the port.
            assert_eq!(ntohs(0x7FFF_5000u32 as i32), 80);
        }

        #[test]
        fn refresh_proc_table_macos_populates() {
            refresh_proc_table_macos();
            // After refresh, cache should exist (may be empty if no sockets / no priv)
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

        #[test]
        fn parse_proc_net_port_hex_max_u16() {
            assert_eq!(parse_proc_net_port("0100007F:FFFF"), Some(65535));
        }

        #[test]
        fn parse_proc_net_port_hex_zero_port() {
            assert_eq!(parse_proc_net_port("00000000:0000"), Some(0));
        }

        #[test]
        fn parse_proc_net_port_ephemeral_hex() {
            assert_eq!(parse_proc_net_port("0100007F:C000"), Some(49152));
        }

        #[test]
        fn parse_proc_net_port_hex_accepts_lowercase() {
            assert_eq!(parse_proc_net_port("0100007F:00ff"), Some(255));
        }

        #[test]
        fn parse_proc_net_port_port_hex_overflow_u16_returns_none() {
            assert_eq!(parse_proc_net_port("0100007F:10000"), None);
        }
    }
}
