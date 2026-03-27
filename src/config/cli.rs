use std::io;
use std::net::IpAddr;

use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};

/// Print the cyberpunk-colorized help and exit.
pub fn print_cyberpunk_help() {
    let v = env!("CARGO_PKG_VERSION");
    print!("\
\x1b[36m ██╗███████╗████████╗ ██████╗ ██████╗ ██████╗ ███████╗\x1b[0m
\x1b[36m ██║██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗██╔══██╗██╔════╝\x1b[0m
\x1b[35m ██║█████╗     ██║   ██║   ██║██████╔╝██████╔╝╚█████╗\x1b[0m
\x1b[35m ██║██╔══╝     ██║   ██║   ██║██╔═══╝ ██╔══██╗ ╚═══██╗\x1b[0m
\x1b[31m ██║██║        ██║   ╚██████╔╝██║     ██║  ██║██████╔╝\x1b[0m
\x1b[31m ╚═╝╚═╝        ╚═╝    ╚═════╝ ╚═╝     ╚═╝  ╚═╝╚═════╝\x1b[0m
\x1b[36m ┌──────────────────────────────────────────────────────┐\x1b[0m
\x1b[36m │\x1b[0m STATUS: \x1b[32mONLINE\x1b[0m  // SIGNAL: \x1b[32m████████\x1b[31m░░\x1b[0m // \x1b[35mv{v}\x1b[0m   \x1b[36m│\x1b[0m
\x1b[36m └──────────────────────────────────────────────────────┘\x1b[0m
\x1b[33m  >> REAL-TIME BANDWIDTH MONITOR // PACKET SNIFFER <<\x1b[0m


Real-time bandwidth monitor (iftop clone in Rust)

\x1b[33m  USAGE:\x1b[0m iftoprs [OPTIONS]

\x1b[36m  ── CAPTURE ────────────────────────────────────────────\x1b[0m
  -i, --interface <INTERFACE>    \x1b[32m//\x1b[0m Network interface to jack into
  -f, --filter <FILTER>          \x1b[32m//\x1b[0m BPF filter expression (e.g., \"tcp port 80\")
  -F, --net-filter <NET_FILTER>  \x1b[32m//\x1b[0m IPv4 network filter in CIDR (e.g., \"192.168.1.0/24\")
  -n, --no-dns                   \x1b[32m//\x1b[0m Kill DNS hostname resolution
  -N, --no-port-names            \x1b[32m//\x1b[0m Kill port-to-service name resolution
  -p, --promiscuous              \x1b[32m//\x1b[0m Enable promiscuous mode ── sniff all traffic on segment
  -b, --no-bars                  \x1b[32m//\x1b[0m Flatline the bar graph display
  -B, --bytes                    \x1b[32m//\x1b[0m Display bandwidth in bytes instead of bits
  -P, --hide-ports               \x1b[32m//\x1b[0m Ghost the port numbers from host display
  -Z, --no-processes             \x1b[32m//\x1b[0m Hide owning process column (shown by default)
  -l, --list-interfaces          \x1b[32m//\x1b[0m Enumerate available interfaces and disconnect
      --list-colors              \x1b[32m//\x1b[0m Preview all 31 color themes
  -h, --help                     Print help
  -V, --version                  Print version
\x1b[36m  ── KEYBINDS ───────────────────────────────────────────\x1b[0m
\x1b[33m  h\x1b[0m ── help HUD       \x1b[33mn\x1b[0m ── toggle DNS      \x1b[33mb\x1b[0m ── bars
\x1b[33m  B\x1b[0m ── bytes/bits     \x1b[33mp\x1b[0m ── ports           \x1b[33mZ\x1b[0m ── processes
\x1b[33m  t\x1b[0m ── line mode      \x1b[33mT\x1b[0m ── cumulative      \x1b[33mP\x1b[0m ── pause
\x1b[33m  x\x1b[0m ── border          \x1b[33mc\x1b[0m ── themes           \x1b[33m/\x1b[0m ── filter
\x1b[33m  g\x1b[0m ── header bar     \x1b[33mf\x1b[0m ── refresh rate
\x1b[33m  1/2/3\x1b[0m ── sort by 2s/10s/40s average
\x1b[33m  < / >\x1b[0m ── sort by src/dst    \x1b[33mo\x1b[0m ── freeze order
\x1b[33m  j/k\x1b[0m ── scroll                \x1b[33mq\x1b[0m ── disconnect

\x1b[36m  ── SYSTEM ─────────────────────────────────────────\x1b[0m
\x1b[35m  v{v}\x1b[0m // \x1b[33m(c) MenkeTechnologies\x1b[0m
\x1b[35m  The packets flow through the wire like neon rain.\x1b[0m
\x1b[33m  >>> JACK IN. SNIFF THE STREAM. OWN YOUR NETWORK. <<<\x1b[0m
\x1b[36m ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░\x1b[0m
");
}

#[derive(Parser, Debug, Clone)]
#[command(
    name = "iftoprs",
    version,
    about = "Real-time bandwidth monitor (iftop clone in Rust)",
    disable_help_flag = true,
    disable_version_flag = true,
)]
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

    /// Hide ports alongside hosts
    #[arg(short = 'P', long = "hide-ports")]
    pub hide_ports: bool,

    /// Hide owning process column
    #[arg(short = 'Z', long = "no-processes")]
    pub no_processes: bool,

    /// List available interfaces and exit
    #[arg(short = 'l', long = "list-interfaces")]
    pub list_interfaces: bool,

    /// Preview all color themes and exit
    #[arg(long = "list-colors")]
    pub list_colors: bool,

    /// Generate shell completions (bash, zsh, fish, elvish, powershell)
    #[arg(long = "completions", value_name = "SHELL")]
    pub completions: Option<Shell>,

    /// Print help
    #[arg(short = 'h', long = "help")]
    pub help: bool,

    /// Print version
    #[arg(short = 'V', long = "version")]
    pub version: bool,
}

impl Args {
    /// Print all theme previews with color swatches.
    pub fn print_colors() {
        use crate::config::theme::ThemeName;
        const RST: &str = "\x1b[0m";
        const B_CYAN: &str = "\x1b[1;36m";
        const B_GREEN: &str = "\x1b[1;32m";
        const B_MAGENTA: &str = "\x1b[1;35m";
        const B_YELLOW: &str = "\x1b[1;33m";

        println!("\n{B_CYAN}  ── BUILTIN COLOR SCHEMES ────────────────────────{RST}\n");
        for &name in ThemeName::ALL {
            let swatch: String = crate::config::theme::Theme::swatch(name)
                .iter()
                .map(|(color, _)| {
                    let idx = match color {
                        ratatui::style::Color::Indexed(n) => *n,
                        _ => 0,
                    };
                    format!("\x1b[48;5;{idx}m   {RST}")
                })
                .collect();
            let flag: String = format!("{:?}", name).to_lowercase();
            println!(
                "  {B_GREEN}{flag:<16}{RST} {B_MAGENTA}{name:<16}{RST} {swatch}",
                name = name.display_name(),
            );
        }
        println!("\n  {B_YELLOW}Usage:{RST} iftoprs then press {B_GREEN}c{RST} for theme chooser");
        println!("  {B_YELLOW}Cycle:{RST} press {B_GREEN}c{RST} in the TUI\n");
    }

    /// Generate shell completions and write to stdout.
    pub fn generate_completions(shell: Shell) {
        let mut cmd = Args::command();
        generate(shell, &mut cmd, "iftoprs", &mut io::stdout());
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn args_with_net_filter(filter: &str) -> Args {
        Args {
            interface: None,
            filter: None,
            net_filter: Some(filter.to_string()),
            no_dns: false,
            no_port_names: false,
            promiscuous: false,
            no_bars: false,
            bytes: false,
            hide_ports: false,
            no_processes: false,
            list_interfaces: false,
            list_colors: false,
            completions: None,
            help: false,
            version: false,
        }
    }

    #[test]
    fn parse_valid_cidr_v4() {
        let args = args_with_net_filter("192.168.1.0/24");
        let (addr, prefix) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "192.168.1.0".parse::<IpAddr>().unwrap());
        assert_eq!(prefix, 24);
    }

    #[test]
    fn parse_valid_cidr_v4_slash8() {
        let args = args_with_net_filter("10.0.0.0/8");
        let (addr, prefix) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "10.0.0.0".parse::<IpAddr>().unwrap());
        assert_eq!(prefix, 8);
    }

    #[test]
    fn parse_invalid_cidr_no_slash() {
        let args = args_with_net_filter("192.168.1.0");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_invalid_cidr_bad_ip() {
        let args = args_with_net_filter("not.an.ip/24");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_invalid_cidr_bad_prefix() {
        let args = args_with_net_filter("10.0.0.0/abc");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_no_net_filter() {
        let args = Args {
            interface: None,
            filter: None,
            net_filter: None,
            no_dns: false,
            no_port_names: false,
            promiscuous: false,
            no_bars: false,
            bytes: false,
            hide_ports: false,
            no_processes: false,
            list_interfaces: false,
            list_colors: false,
            completions: None,
            help: false,
            version: false,
        };
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn clap_command_builds() {
        let cmd = Args::command();
        assert_eq!(cmd.get_name(), "iftoprs");
    }
}
