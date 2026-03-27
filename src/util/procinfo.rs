use std::net::IpAddr;
use crate::data::flow::Protocol;

/// Look up the owning process for a network flow.
/// Uses lsof on macOS for reliable socket-to-PID mapping.
#[cfg(target_os = "macos")]
pub fn lookup_process(
    _src: IpAddr,
    src_port: u16,
    _dst: IpAddr,
    dst_port: u16,
    protocol: &Protocol,
) -> Option<(u32, String)> {
    // Only TCP/UDP
    let proto_flag = match protocol {
        Protocol::Tcp => "TCP",
        Protocol::Udp => "UDP",
        _ => return None,
    };

    // Try both ports — one will be local
    for port in [src_port, dst_port] {
        if port == 0 { continue; }
        if let Some(result) = lsof_lookup(port, proto_flag) {
            return Some(result);
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn lsof_lookup(port: u16, proto: &str) -> Option<(u32, String)> {
    use std::process::Command;

    let output = Command::new("lsof")
        .args(["-i", &format!("{}:{}", proto, port), "-n", "-P", "-F", "pc"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut pid: Option<u32> = None;
    let mut name: Option<String> = None;

    for line in stdout.lines() {
        if let Some(p) = line.strip_prefix('p') {
            pid = p.parse().ok();
        } else if let Some(n) = line.strip_prefix('c') {
            name = Some(n.to_string());
        }
        if pid.is_some() && name.is_some() {
            return Some((pid.unwrap(), name.unwrap()));
        }
    }
    None
}

#[cfg(not(target_os = "macos"))]
pub fn lookup_process(
    _src: IpAddr,
    _src_port: u16,
    _dst: IpAddr,
    _dst_port: u16,
    _protocol: &Protocol,
) -> Option<(u32, String)> {
    None
}
