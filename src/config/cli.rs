use std::net::IpAddr;

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "iftoprs",
    about = "Real-time bandwidth monitor (iftop clone in Rust)",
    long_about = r#"
 ██╗███████╗████████╗ ██████╗ ██████╗ ██████╗ ███████╗
 ██║██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗██╔══██╗██╔════╝
 ██║█████╗     ██║   ██║   ██║██████╔╝██████╔╝╚█████╗
 ██║██╔══╝     ██║   ██║   ██║██╔═══╝ ██╔══██╗ ╚═══██╗
 ██║██║        ██║   ╚██████╔╝██║     ██║  ██║██████╔╝
 ╚═╝╚═╝        ╚═╝    ╚═════╝ ╚═╝     ╚═╝  ╚═╝╚═════╝

  [ SYSTEM://NET_INTERCEPT v1.0 ]
  ⟦ JACKING INTO YOUR PACKET STREAM ⟧

  A neon-drenched terminal UI for real-time bandwidth monitoring.
  Captures live network traffic via libpcap and renders per-flow
  bandwidth with sliding-window averages on a log10 scale.

  ── CAPTURE ──────────────────────────────────────────
  Sniffs raw packets on the specified interface (or auto-
  detects the default gateway). Supports BPF filters,
  CIDR network filters, and promiscuous mode.

  ── DISPLAY ──────────────────────────────────────────
  Color-coded rate columns: yellow(2s) / green(10s) /
  cyan(40s). Bar graphs, DNS resolution, port-to-service
  mapping, and process attribution toggleable at runtime.

  ── KEYBINDS ─────────────────────────────────────────
  h ── help HUD       n ── toggle DNS      b ── bars
  B ── bytes/bits     p ── ports           Z ── processes
  t ── line mode      T ── cumulative      P ── pause
  1/2/3 ── sort by 2s/10s/40s average
  < / > ── sort by src/dst    o ── freeze order
  j/k ── scroll                q ── disconnect

  // THE STREET FINDS ITS OWN USES FOR BANDWIDTH //
"#,
    after_help = "⟦ END OF LINE ⟧"
)]
pub struct Args {
    /// >> Network interface to jack into
    #[arg(short = 'i', long)]
    pub interface: Option<String>,

    /// >> BPF filter expression (e.g., "tcp port 80")
    #[arg(short = 'f', long)]
    pub filter: Option<String>,

    /// >> IPv4 network filter in CIDR (e.g., "192.168.1.0/24")
    #[arg(short = 'F', long = "net-filter")]
    pub net_filter: Option<String>,

    /// >> Kill DNS hostname resolution
    #[arg(short = 'n', long = "no-dns")]
    pub no_dns: bool,

    /// >> Kill port-to-service name resolution
    #[arg(short = 'N', long = "no-port-names")]
    pub no_port_names: bool,

    /// >> Enable promiscuous mode ── sniff all traffic on segment
    #[arg(short = 'p', long)]
    pub promiscuous: bool,

    /// >> Flatline the bar graph display
    #[arg(short = 'b', long = "no-bars")]
    pub no_bars: bool,

    /// >> Display bandwidth in bytes instead of bits
    #[arg(short = 'B', long = "bytes")]
    pub bytes: bool,

    /// >> Ghost the port numbers from host display
    #[arg(short = 'P', long = "hide-ports")]
    pub hide_ports: bool,

    /// >> Expose owning process for each flow
    #[arg(short = 'Z', long = "show-processes")]
    pub show_processes: bool,

    /// >> Enumerate available interfaces and disconnect
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

}
