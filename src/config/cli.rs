use std::net::IpAddr;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "iftoprs", about = "Real-time bandwidth monitor (iftop clone in Rust)")]
pub struct Args {
    /// Network interface to monitor
    #[arg(short = 'i', long)]
    pub interface: Option<String>,

    /// BPF filter expression (e.g., "tcp port 80")
    #[arg(short = 'f', long)]
    pub filter: Option<String>,

    /// IPv4 network filter (e.g., "192.168.1.0/24")
    #[arg(short = 'F', long = "net-filter")]
    pub net_filter: Option<String>,

    /// Disable DNS hostname resolution
    #[arg(short = 'n', long = "no-dns")]
    pub no_dns: bool,

    /// Disable port-to-service resolution
    #[arg(short = 'N', long = "no-port-names")]
    pub no_port_names: bool,

    /// Enable promiscuous mode
    #[arg(short = 'p', long)]
    pub promiscuous: bool,

    /// Disable bar graph display
    #[arg(short = 'b', long = "no-bars")]
    pub no_bars: bool,

    /// Display bandwidth in bytes (instead of bits)
    #[arg(short = 'B', long = "bytes")]
    pub bytes: bool,

    /// Hide ports alongside hosts (ports shown by default)
    #[arg(short = 'P', long = "hide-ports")]
    pub hide_ports: bool,

    /// Show owning process for each flow
    #[arg(short = 'Z', long = "show-processes")]
    pub show_processes: bool,

    /// Set bandwidth scale ceiling (e.g., "10M", "1G")
    #[arg(short = 'm', long = "max-bandwidth")]
    pub max_bandwidth: Option<String>,

    /// List available interfaces and exit
    #[arg(short = 'l', long = "list-interfaces")]
    pub list_interfaces: bool,
}

impl Args {
    /// Parse the -F net filter into (network_addr, prefix_len).
    pub fn parse_net_filter(&self) -> Option<(IpAddr, u8)> {
        let filter = self.net_filter.as_ref()?;
        let parts: Vec<&str> = filter.split('/').collect();
        if parts.len() != 2 {
            eprintln!("Warning: invalid net filter '{}', expected CIDR notation", filter);
            return None;
        }
        let addr: IpAddr = parts[0].parse().ok()?;
        let prefix: u8 = parts[1].parse().ok()?;
        Some((addr, prefix))
    }

    /// Parse max bandwidth string like "10M", "1G", "500K" into bytes/sec.
    pub fn parse_max_bandwidth(&self) -> Option<f64> {
        let s = self.max_bandwidth.as_ref()?;
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        let (num_str, multiplier) = if s.ends_with('G') || s.ends_with('g') {
            (&s[..s.len() - 1], 1_000_000_000.0)
        } else if s.ends_with('M') || s.ends_with('m') {
            (&s[..s.len() - 1], 1_000_000.0)
        } else if s.ends_with('K') || s.ends_with('k') {
            (&s[..s.len() - 1], 1_000.0)
        } else {
            (s, 1.0)
        };
        let num: f64 = num_str.parse().ok()?;
        Some(num * multiplier)
    }
}
