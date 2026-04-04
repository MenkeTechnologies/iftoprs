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
  -c, --config <FILE>            \x1b[32m//\x1b[0m Path to config file (default: ~/.iftoprs.conf)
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
      --json                     \x1b[32m//\x1b[0m Stream NDJSON to stdout (no TUI)
  -l, --list-interfaces          \x1b[32m//\x1b[0m Enumerate available interfaces and disconnect
      --list-colors              \x1b[32m//\x1b[0m Preview all 31 color themes
  -h, --help                     Print help
  -V, --version                  Print version
\x1b[36m  ── KEYBINDS ───────────────────────────────────────────\x1b[0m
\x1b[33m  h\x1b[0m ── help HUD       \x1b[33mn\x1b[0m ── toggle DNS      \x1b[33mb\x1b[0m ── bars
\x1b[33m  B\x1b[0m ── bytes/bits     \x1b[33mp\x1b[0m ── ports           \x1b[33mZ\x1b[0m ── processes
\x1b[33m  t\x1b[0m ── line mode      \x1b[33mT\x1b[0m ── hover tips      \x1b[33mP\x1b[0m ── pause
\x1b[33m  U\x1b[0m ── cumulative
\x1b[33m  x\x1b[0m ── border          \x1b[33mc\x1b[0m ── themes           \x1b[33m/\x1b[0m ── filter
\x1b[33m  g\x1b[0m ── header bar     \x1b[33mf\x1b[0m ── refresh rate    \x1b[33mTab\x1b[0m ── switch view
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
    disable_version_flag = true
)]
pub struct Args {
    /// Path to config file (default: ~/.iftoprs.conf)
    #[arg(short = 'c', long = "config")]
    pub config: Option<String>,

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

    /// Stream flow data as newline-delimited JSON (no TUI)
    #[arg(long = "json")]
    pub json: bool,

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
            eprintln!(
                "Warning: invalid net filter '{}', expected CIDR notation",
                filter
            );
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
            config: None,
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
            json: false,
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
            config: None,
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
            json: false,
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

    #[test]
    fn parse_valid_cidr_v6() {
        let args = args_with_net_filter("2001:db8::/32");
        let (addr, prefix) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "2001:db8::".parse::<IpAddr>().unwrap());
        assert_eq!(prefix, 32);
    }

    #[test]
    fn parse_valid_cidr_v6_host() {
        let args = args_with_net_filter("::1/128");
        let (addr, prefix) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "::1".parse::<IpAddr>().unwrap());
        assert_eq!(prefix, 128);
    }

    #[test]
    fn parse_cidr_with_double_slash_invalid() {
        let args = args_with_net_filter("10.0.0.0/24/8");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_cidr_slash_zero() {
        let args = args_with_net_filter("0.0.0.0/0");
        let (_, prefix) = args.parse_net_filter().unwrap();
        assert_eq!(prefix, 0);
    }

    #[test]
    fn parse_cidr_slash_32() {
        let args = args_with_net_filter("10.0.0.1/32");
        let (addr, prefix) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "10.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(prefix, 32);
    }

    #[test]
    fn parse_cidr_empty_string() {
        let args = args_with_net_filter("");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_cidr_only_slash() {
        let args = args_with_net_filter("/");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn default_args_all_false() {
        let args = Args {
            config: None,
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
            json: false,
            list_interfaces: false,
            list_colors: false,
            completions: None,
            help: false,
            version: false,
        };
        assert!(!args.no_dns);
        assert!(!args.json);
        assert!(!args.help);
        assert!(!args.version);
        assert!(args.config.is_none());
        assert!(args.interface.is_none());
        assert!(args.filter.is_none());
    }

    #[test]
    fn clap_command_has_all_flags() {
        let cmd = Args::command();
        let args: Vec<String> = cmd
            .get_arguments()
            .map(|a| a.get_id().to_string())
            .collect();
        for flag in [
            "interface",
            "filter",
            "no_dns",
            "no_bars",
            "bytes",
            "hide_ports",
            "no_processes",
            "json",
            "list_interfaces",
            "list_colors",
            "completions",
            "help",
            "version",
            "config",
            "net_filter",
            "promiscuous",
            "no_port_names",
        ] {
            assert!(args.contains(&flag.to_string()), "missing flag: {}", flag);
        }
    }

    #[test]
    fn clap_parse_help_flag() {
        let args = Args::try_parse_from(["iftoprs", "-h"]).unwrap();
        assert!(args.help);
    }

    #[test]
    fn clap_parse_version_flag() {
        let args = Args::try_parse_from(["iftoprs", "-V"]).unwrap();
        assert!(args.version);
    }

    #[test]
    fn clap_parse_json_flag() {
        let args = Args::try_parse_from(["iftoprs", "--json"]).unwrap();
        assert!(args.json);
    }

    #[test]
    fn clap_parse_interface_short() {
        let args = Args::try_parse_from(["iftoprs", "-i", "en0"]).unwrap();
        assert_eq!(args.interface, Some("en0".to_string()));
    }

    #[test]
    fn clap_parse_config_short() {
        let args = Args::try_parse_from(["iftoprs", "-c", "/tmp/test.conf"]).unwrap();
        assert_eq!(args.config, Some("/tmp/test.conf".to_string()));
    }

    #[test]
    fn clap_parse_multiple_flags() {
        let args = Args::try_parse_from(["iftoprs", "-n", "-B", "-b", "--json"]).unwrap();
        assert!(args.no_dns);
        assert!(args.bytes);
        assert!(args.no_bars);
        assert!(args.json);
    }

    #[test]
    fn clap_parse_filter() {
        let args = Args::try_parse_from(["iftoprs", "-f", "tcp port 80"]).unwrap();
        assert_eq!(args.filter, Some("tcp port 80".to_string()));
    }

    #[test]
    fn clap_parse_net_filter() {
        let args = Args::try_parse_from(["iftoprs", "-F", "10.0.0.0/8"]).unwrap();
        assert_eq!(args.net_filter, Some("10.0.0.0/8".to_string()));
    }

    #[test]
    fn clap_parse_completions() {
        let args = Args::try_parse_from(["iftoprs", "--completions", "zsh"]).unwrap();
        assert_eq!(args.completions, Some(Shell::Zsh));
    }

    #[test]
    fn clap_parse_completions_bash() {
        let args = Args::try_parse_from(["iftoprs", "--completions", "bash"]).unwrap();
        assert_eq!(args.completions, Some(Shell::Bash));
    }

    #[test]
    fn clap_parse_completions_fish() {
        let args = Args::try_parse_from(["iftoprs", "--completions", "fish"]).unwrap();
        assert_eq!(args.completions, Some(Shell::Fish));
    }

    #[test]
    fn clap_parse_completions_elvish() {
        let args = Args::try_parse_from(["iftoprs", "--completions", "elvish"]).unwrap();
        assert_eq!(args.completions, Some(Shell::Elvish));
    }

    #[test]
    fn clap_parse_completions_powershell() {
        let args = Args::try_parse_from(["iftoprs", "--completions", "powershell"]).unwrap();
        assert_eq!(args.completions, Some(Shell::PowerShell));
    }

    #[test]
    fn clap_parse_long_help() {
        let args = Args::try_parse_from(["iftoprs", "--help"]).unwrap();
        assert!(args.help);
    }

    #[test]
    fn clap_parse_long_version() {
        let args = Args::try_parse_from(["iftoprs", "--version"]).unwrap();
        assert!(args.version);
    }

    #[test]
    fn clap_parse_list_interfaces() {
        let args = Args::try_parse_from(["iftoprs", "--list-interfaces"]).unwrap();
        assert!(args.list_interfaces);
    }

    #[test]
    fn clap_parse_list_interfaces_short() {
        let args = Args::try_parse_from(["iftoprs", "-l"]).unwrap();
        assert!(args.list_interfaces);
        assert!(!args.list_colors);
    }

    #[test]
    fn clap_parse_list_colors() {
        let args = Args::try_parse_from(["iftoprs", "--list-colors"]).unwrap();
        assert!(args.list_colors);
        assert!(!args.list_interfaces);
    }

    #[test]
    fn clap_parse_hide_ports() {
        let args = Args::try_parse_from(["iftoprs", "--hide-ports"]).unwrap();
        assert!(args.hide_ports);
    }

    #[test]
    fn clap_parse_hide_ports_short() {
        let args = Args::try_parse_from(["iftoprs", "-P"]).unwrap();
        assert!(args.hide_ports);
    }

    #[test]
    fn clap_parse_no_port_names() {
        let args = Args::try_parse_from(["iftoprs", "--no-port-names"]).unwrap();
        assert!(args.no_port_names);
    }

    #[test]
    fn clap_parse_promiscuous() {
        let args = Args::try_parse_from(["iftoprs", "--promiscuous"]).unwrap();
        assert!(args.promiscuous);
    }

    #[test]
    fn clap_parse_promiscuous_short() {
        let args = Args::try_parse_from(["iftoprs", "-p"]).unwrap();
        assert!(args.promiscuous);
    }

    #[test]
    fn clap_parse_interface_long_equals() {
        let args = Args::try_parse_from(["iftoprs", "--interface=eth0"]).unwrap();
        assert_eq!(args.interface, Some("eth0".into()));
    }

    #[test]
    fn clap_parse_net_filter_long_equals() {
        let args = Args::try_parse_from(["iftoprs", "--net-filter=172.16.0.0/12"]).unwrap();
        assert_eq!(args.net_filter, Some("172.16.0.0/12".into()));
    }

    #[test]
    fn clap_parse_capture_bundle() {
        let args = Args::try_parse_from([
            "iftoprs",
            "-i",
            "wlan0",
            "-f",
            "udp",
            "-F",
            "10.0.0.0/8",
            "-n",
            "-p",
            "-B",
        ])
        .unwrap();
        assert_eq!(args.interface, Some("wlan0".into()));
        assert_eq!(args.filter, Some("udp".into()));
        assert_eq!(args.net_filter, Some("10.0.0.0/8".into()));
        assert!(args.no_dns);
        assert!(args.promiscuous);
        assert!(args.bytes);
    }

    #[test]
    fn parse_valid_cidr_v6_slash48() {
        let args = args_with_net_filter("2001:db8:beef::/48");
        let (addr, prefix) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "2001:db8:beef::".parse::<IpAddr>().unwrap());
        assert_eq!(prefix, 48);
    }

    #[test]
    fn parse_cidr_trailing_slash_invalid() {
        let args = args_with_net_filter("10.0.0.0/");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_cidr_double_slash_invalid() {
        let args = args_with_net_filter("10.0.0.0//24");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn clap_parse_no_processes() {
        let args = Args::try_parse_from(["iftoprs", "--no-processes"]).unwrap();
        assert!(args.no_processes);
    }

    #[test]
    fn clap_parse_no_processes_short() {
        let args = Args::try_parse_from(["iftoprs", "-Z"]).unwrap();
        assert!(args.no_processes);
    }

    #[test]
    fn clap_rejects_unknown_shell_for_completions() {
        assert!(Args::try_parse_from(["iftoprs", "--completions", "notashell"]).is_err());
    }

    #[test]
    fn parse_valid_cidr_ipv4_loopback() {
        let args = args_with_net_filter("127.0.0.0/8");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "127.0.0.0".parse::<IpAddr>().unwrap());
        assert_eq!(p, 8);
    }

    #[test]
    fn parse_valid_cidr_multicast() {
        let args = args_with_net_filter("224.0.0.0/4");
        let (_, p) = args.parse_net_filter().unwrap();
        assert_eq!(p, 4);
    }

    #[test]
    fn clap_parse_filter_long_equals() {
        let args = Args::try_parse_from(["iftoprs", "--filter=udp port 53"]).unwrap();
        assert_eq!(args.filter, Some("udp port 53".into()));
    }

    #[test]
    fn clap_parse_json_with_other_flags() {
        let args = Args::try_parse_from(["iftoprs", "--json", "-n", "-B"]).unwrap();
        assert!(args.json);
        assert!(args.no_dns);
        assert!(args.bytes);
    }

    #[test]
    fn parse_cidr_ipv6_loopback() {
        let args = args_with_net_filter("::1/128");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "::1".parse::<IpAddr>().unwrap());
        assert_eq!(p, 128);
    }

    #[test]
    fn parse_cidr_ipv6_ula_fd00_slash8() {
        let args = args_with_net_filter("fd00::/8");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "fd00::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 8);
    }

    #[test]
    fn parse_cidr_ipv4_class_c() {
        let args = args_with_net_filter("203.0.113.0/24");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "203.0.113.0".parse::<IpAddr>().unwrap());
        assert_eq!(p, 24);
    }

    #[test]
    fn clap_parse_no_dns_and_interface_separate() {
        let args = Args::try_parse_from(["iftoprs", "-n", "-i", "lo0"]).unwrap();
        assert!(args.no_dns);
        assert_eq!(args.interface, Some("lo0".into()));
    }

    #[test]
    fn clap_parse_net_filter_short_capital_f() {
        let args = Args::try_parse_from(["iftoprs", "-F", "fe80::/10"]).unwrap();
        assert_eq!(args.net_filter, Some("fe80::/10".into()));
    }

    #[test]
    fn clap_parse_long_help_only() {
        let args = Args::try_parse_from(["iftoprs", "--help"]).unwrap();
        assert!(args.help);
        assert!(!args.version);
    }

    #[test]
    fn clap_parse_long_version_only() {
        let args = Args::try_parse_from(["iftoprs", "--version"]).unwrap();
        assert!(args.version);
        assert!(!args.help);
    }

    #[test]
    fn parse_cidr_non_numeric_prefix_invalid() {
        let args = args_with_net_filter("10.0.0.0/xx");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn clap_parse_config_long_equals() {
        let args = Args::try_parse_from(["iftoprs", "--config=/tmp/x.toml"]).unwrap();
        assert_eq!(args.config, Some("/tmp/x.toml".into()));
    }

    #[test]
    fn clap_parse_hide_ports_and_no_bars() {
        let args = Args::try_parse_from(["iftoprs", "--hide-ports", "--no-bars"]).unwrap();
        assert!(args.hide_ports);
        assert!(args.no_bars);
    }

    #[test]
    fn parse_cidr_ipv4_host_max_slash32() {
        let args = args_with_net_filter("192.0.2.255/32");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "192.0.2.255".parse::<IpAddr>().unwrap());
        assert_eq!(p, 32);
    }

    #[test]
    fn clap_parse_promiscuous_and_no_dns() {
        let args = Args::try_parse_from(["iftoprs", "-p", "-n"]).unwrap();
        assert!(args.promiscuous);
        assert!(args.no_dns);
    }

    #[test]
    fn clap_parse_list_interfaces_and_list_colors() {
        let args = Args::try_parse_from(["iftoprs", "-l", "--list-colors"]).unwrap();
        assert!(args.list_interfaces);
        assert!(args.list_colors);
    }

    #[test]
    fn parse_cidr_ipv6_ula_slash48() {
        let args = args_with_net_filter("fd00::/48");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "fd00::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 48);
    }

    #[test]
    fn clap_parse_bytes_and_no_dns() {
        let args = Args::try_parse_from(["iftoprs", "-B", "-n"]).unwrap();
        assert!(args.bytes);
        assert!(args.no_dns);
    }

    #[test]
    fn clap_parse_json_and_no_processes() {
        let args = Args::try_parse_from(["iftoprs", "--json", "-Z"]).unwrap();
        assert!(args.json);
        assert!(args.no_processes);
    }

    #[test]
    fn clap_parse_no_port_names_short() {
        let args = Args::try_parse_from(["iftoprs", "-N"]).unwrap();
        assert!(args.no_port_names);
    }

    #[test]
    fn clap_parse_filter_expression() {
        let args = Args::try_parse_from(["iftoprs", "-f", "tcp port 443"]).unwrap();
        assert_eq!(args.filter.as_deref(), Some("tcp port 443"));
    }

    #[test]
    fn clap_parse_promiscuous_bytes_hide_ports() {
        let args = Args::try_parse_from(["iftoprs", "-p", "-B", "-P"]).unwrap();
        assert!(args.promiscuous);
        assert!(args.bytes);
        assert!(args.hide_ports);
    }

    #[test]
    fn clap_parse_config_short_interface() {
        let args = Args::try_parse_from(["iftoprs", "-c", "/tmp/c.toml", "-i", "eth0"]).unwrap();
        assert_eq!(args.config.as_deref(), Some("/tmp/c.toml"));
        assert_eq!(args.interface.as_deref(), Some("eth0"));
    }

    #[test]
    fn parse_cidr_ipv4_multicast_slash24() {
        let args = args_with_net_filter("224.0.0.0/24");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "224.0.0.0".parse::<IpAddr>().unwrap());
        assert_eq!(p, 24);
    }

    #[test]
    fn parse_cidr_ipv6_link_local_fe80_slash10() {
        let args = args_with_net_filter("fe80::/10");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "fe80::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 10);
    }

    #[test]
    fn parse_cidr_ipv6_global_unicast_2001_db8_slash32() {
        let args = args_with_net_filter("2001:db8::/32");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "2001:db8::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 32);
    }

    #[test]
    fn clap_parse_filter_long_equals_icmp() {
        let args = Args::try_parse_from(["iftoprs", "--filter=icmp"]).unwrap();
        assert_eq!(args.filter.as_deref(), Some("icmp"));
    }

    #[test]
    fn clap_parse_no_bars_short_long_combo() {
        let args = Args::try_parse_from(["iftoprs", "-b", "--bytes"]).unwrap();
        assert!(args.no_bars);
        assert!(args.bytes);
    }

    #[test]
    fn parse_cidr_ipv4_private_class_b() {
        let args = args_with_net_filter("172.20.0.0/16");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "172.20.0.0".parse::<IpAddr>().unwrap());
        assert_eq!(p, 16);
    }

    #[test]
    fn parse_cidr_ipv6_unique_local_fd_slash8() {
        let args = args_with_net_filter("fd00::/8");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "fd00::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 8);
    }

    #[test]
    fn parse_cidr_ipv4_mapped_ipv6_host_slash128() {
        let args = args_with_net_filter("::ffff:192.0.2.1/128");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "::ffff:192.0.2.1".parse::<IpAddr>().unwrap());
        assert_eq!(p, 128);
    }

    #[test]
    fn clap_parse_hide_ports_with_list_colors() {
        let args = Args::try_parse_from(["iftoprs", "-P", "--list-colors"]).unwrap();
        assert!(args.hide_ports);
        assert!(args.list_colors);
    }

    #[test]
    fn clap_parse_json_no_port_names_no_dns() {
        let args = Args::try_parse_from(["iftoprs", "--json", "-N", "-n"]).unwrap();
        assert!(args.json);
        assert!(args.no_port_names);
        assert!(args.no_dns);
    }

    #[test]
    fn parse_cidr_ipv6_global_slash56() {
        let args = args_with_net_filter("2600::/56");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "2600::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 56);
    }

    #[test]
    fn clap_parse_interface_short_equals_form() {
        let args = Args::try_parse_from(["iftoprs", "-i=en0"]).unwrap();
        assert_eq!(args.interface.as_deref(), Some("en0"));
    }

    #[test]
    fn parse_cidr_ipv4_cgnat_slash10() {
        let args = args_with_net_filter("100.64.0.0/10");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "100.64.0.0".parse::<IpAddr>().unwrap());
        assert_eq!(p, 10);
    }

    #[test]
    fn clap_parse_no_bars_no_dns_combo() {
        let args = Args::try_parse_from(["iftoprs", "-b", "-n", "-N"]).unwrap();
        assert!(args.no_bars);
        assert!(args.no_dns);
        assert!(args.no_port_names);
    }

    #[test]
    fn parse_cidr_ipv6_slash48_site_prefix() {
        let args = args_with_net_filter("2001:db8::/48");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "2001:db8::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 48);
    }

    #[test]
    fn parse_cidr_ipv6_multicast_ff00_slash8() {
        let args = args_with_net_filter("ff00::/8");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "ff00::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 8);
    }

    #[test]
    fn clap_parse_bytes_short_only() {
        let args = Args::try_parse_from(["iftoprs", "-B"]).unwrap();
        assert!(args.bytes);
        assert!(!args.json);
    }

    #[test]
    fn clap_parse_promiscuous_list_interfaces() {
        let args = Args::try_parse_from(["iftoprs", "-p", "-l"]).unwrap();
        assert!(args.promiscuous);
        assert!(args.list_interfaces);
    }

    #[test]
    fn parse_cidr_ipv4_link_local_169_slash16() {
        let args = args_with_net_filter("169.254.0.0/16");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "169.254.0.0".parse::<IpAddr>().unwrap());
        assert_eq!(p, 16);
    }

    #[test]
    fn clap_parse_config_file_with_filter() {
        let args =
            Args::try_parse_from(["iftoprs", "-c", "/path/prefs.toml", "-f", "host 192.0.2.1"])
                .unwrap();
        assert_eq!(args.config.as_deref(), Some("/path/prefs.toml"));
        assert_eq!(args.filter.as_deref(), Some("host 192.0.2.1"));
    }

    #[test]
    fn clap_parse_hide_processes_and_hide_ports() {
        let args = Args::try_parse_from(["iftoprs", "-Z", "-P"]).unwrap();
        assert!(args.no_processes);
        assert!(args.hide_ports);
    }

    #[test]
    fn clap_parse_json_list_interfaces_exits_flags() {
        let args = Args::try_parse_from(["iftoprs", "--json", "-l"]).unwrap();
        assert!(args.json);
        assert!(args.list_interfaces);
    }

    #[test]
    fn parse_cidr_ipv4_slash30_point_to_point() {
        let args = args_with_net_filter("192.0.2.0/30");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "192.0.2.0".parse::<IpAddr>().unwrap());
        assert_eq!(p, 30);
    }

    #[test]
    fn clap_parse_version_short_flag_with_interface() {
        let args = Args::try_parse_from(["iftoprs", "-V", "-i", "lo"]).unwrap();
        assert!(args.version);
        assert_eq!(args.interface.as_deref(), Some("lo"));
    }

    #[test]
    fn parse_net_filter_multiple_slashes_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24/32");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_empty_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_cidr_ipv4_broadcast_slash32() {
        let args = args_with_net_filter("255.255.255.255/32");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "255.255.255.255".parse::<IpAddr>().unwrap());
        assert_eq!(p, 32);
    }

    #[test]
    fn parse_net_filter_missing_slash_returns_none() {
        let args = args_with_net_filter("10.0.0.0");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_non_numeric_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24abc");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_ipv6_address_without_slash_returns_none() {
        let args = args_with_net_filter("2001:db8::1");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_double_slash_returns_none() {
        let args = args_with_net_filter("10.0.0.0//24");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_cidr_ipv6_prefix_127_loopback_pair() {
        let args = args_with_net_filter("fe80::1/127");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "fe80::1".parse::<IpAddr>().unwrap());
        assert_eq!(p, 127);
    }

    #[test]
    fn parse_cidr_ipv4_host_slash32_not_network_zero() {
        let args = args_with_net_filter("192.168.0.1/32");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "192.168.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(p, 32);
    }

    #[test]
    fn parse_cidr_ipv6_unique_local_fc00_slash7() {
        let args = args_with_net_filter("fc00::/7");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "fc00::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 7);
    }

    #[test]
    fn parse_cidr_ipv6_multicast_ff02_link_local_slash16() {
        let args = args_with_net_filter("ff02::/16");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "ff02::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 16);
    }

    #[test]
    fn parse_cidr_ipv6_all_addresses_slash0() {
        let args = args_with_net_filter("::/0");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 0);
    }

    #[test]
    fn parse_cidr_ipv6_benchmark_prefix_2001_2_slash48() {
        let args = args_with_net_filter("2001:2::/48");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "2001:2::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 48);
    }

    #[test]
    fn parse_cidr_ipv6_nat64_well_known_prefix_slash96() {
        let args = args_with_net_filter("64:ff9b::/96");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "64:ff9b::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 96);
    }

    #[test]
    fn parse_cidr_ipv6_teredo_prefix_slash32() {
        let args = args_with_net_filter("2001::/32");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "2001::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 32);
    }

    #[test]
    fn parse_cidr_ipv6_six_to_four_prefix_slash16() {
        let args = args_with_net_filter("2002::/16");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "2002::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 16);
    }

    #[test]
    fn parse_cidr_ipv6_orchid_slash28() {
        let args = args_with_net_filter("2001:10::/28");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "2001:10::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 28);
    }

    #[test]
    fn parse_cidr_ipv6_link_local_slash64() {
        let args = args_with_net_filter("fe80::/64");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "fe80::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 64);
    }

    #[test]
    fn parse_cidr_ipv4_documentation_test_net_2_slash24() {
        let args = args_with_net_filter("198.51.100.0/24");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "198.51.100.0".parse::<IpAddr>().unwrap());
        assert_eq!(p, 24);
    }

    #[test]
    fn parse_cidr_ipv6_documentation_db8_subprefix_slash48() {
        let args = args_with_net_filter("2001:db8:1::/48");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "2001:db8:1::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 48);
    }

    #[test]
    fn parse_cidr_ipv6_ipv4_mapped_well_known_slash96() {
        let args = args_with_net_filter("::ffff:0:0/96");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "::ffff:0:0".parse::<IpAddr>().unwrap());
        assert_eq!(p, 96);
    }

    #[test]
    fn parse_cidr_ipv6_site_local_deprecated_fec0_slash10() {
        let args = args_with_net_filter("fec0::/10");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "fec0::".parse::<IpAddr>().unwrap());
        assert_eq!(p, 10);
    }

    #[test]
    fn parse_cidr_ipv4_prefix_byte_255_parses() {
        let args = args_with_net_filter("10.0.0.5/255");
        let (addr, p) = args.parse_net_filter().unwrap();
        assert_eq!(addr, "10.0.0.5".parse::<IpAddr>().unwrap());
        assert_eq!(p, 255);
    }

    #[test]
    fn parse_net_filter_space_before_slash_returns_none() {
        let args = args_with_net_filter("10.0.0.0 /24");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_space_before_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/ 24");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_cidr_ipv4_prefix_with_leading_zero_decimal() {
        let args = args_with_net_filter("10.0.0.0/024");
        let (_, p) = args.parse_net_filter().unwrap();
        assert_eq!(p, 24);
    }

    #[test]
    fn parse_net_filter_negative_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/-1");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_prefix_256_overflow_returns_none() {
        let args = args_with_net_filter("10.0.0.0/256");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_empty_left_of_slash_returns_none() {
        let args = args_with_net_filter("/24");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_tab_after_ip_returns_none() {
        let args = args_with_net_filter("10.0.0.0\t/24");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_trailing_space_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24 ");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_carriage_return_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\r");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_newline_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\n");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_nul_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_leading_zero_prefix_digits_parse_as_decimal() {
        let args = args_with_net_filter("192.168.0.0/024");
        let (_, p) = args.parse_net_filter().unwrap();
        assert_eq!(p, 24);
    }

    #[test]
    fn parse_net_filter_vertical_tab_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0b}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_form_feed_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0c}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_narrow_no_break_space_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{202f}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_line_separator_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2028}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_paragraph_separator_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2029}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_next_line_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{85}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_no_break_space_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{a0}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_left_to_right_mark_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{200e}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_object_replacement_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{fffd}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_word_joiner_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2060}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_right_to_left_mark_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{200f}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_pop_directional_isolate_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2069}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_invisible_plus_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2064}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_soft_hyphen_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{00ad}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_bom_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{feff}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_left_to_right_isolate_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2066}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_right_to_left_isolate_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2067}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_zero_width_non_joiner_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{200c}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_grapheme_joiner_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{034f}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_zero_width_joiner_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{200d}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_first_strong_isolate_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2068}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_inhibit_symmetric_swapping_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{206a}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_activate_symmetric_swapping_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{206b}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_inhibit_arabic_form_shaping_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{206c}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_activate_arabic_form_shaping_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{206d}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_national_digit_shapes_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{206e}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_nominal_digit_shapes_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{206f}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_zero_width_space_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{200b}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_figure_space_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2007}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_en_space_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2002}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_em_space_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2003}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_punctuation_space_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2008}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_thin_space_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2009}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_hair_space_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{200a}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_en_quad_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2000}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_em_quad_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2001}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_four_per_em_space_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2005}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_three_per_em_space_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2004}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_six_per_em_space_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2006}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_arabic_letter_mark_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{061c}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_mongolian_vowel_separator_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{180e}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_left_to_right_embedding_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{202a}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_right_to_left_embedding_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{202b}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_pop_directional_formatting_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{202c}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_left_to_right_override_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{202d}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_right_to_left_override_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{202e}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_function_application_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2061}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_invisible_times_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2062}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_invisible_separator_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{2063}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_grave_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0300}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_acute_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0301}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_circumflex_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0302}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_tilde_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0303}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_macron_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0304}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_overline_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0305}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_breve_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0306}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_dot_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0307}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_diaeresis_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0308}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_hook_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0309}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_ring_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{030a}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_double_acute_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{030b}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_caron_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{030c}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_vertical_line_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{030d}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_double_vertical_line_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{030e}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_double_grave_accent_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{030f}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_candrabindu_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0310}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_inverted_breve_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0311}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_turned_comma_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0312}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_comma_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0313}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_reversed_comma_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0314}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_comma_above_right_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0315}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_grave_accent_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0316}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_acute_accent_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0317}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_left_tack_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0318}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_right_tack_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0319}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_left_angle_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{031a}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_horn_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{031b}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_left_half_ring_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{031c}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_up_tack_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{031d}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_plus_sign_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{031e}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_minus_sign_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{031f}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_palatalized_hook_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0320}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_retroflex_hook_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0321}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_retroflex_hook_below_u0322_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0322}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_dot_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0323}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_diaeresis_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0324}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_ring_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0325}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_comma_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0326}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_cedilla_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0327}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_ogonek_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0328}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_vertical_line_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0329}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_bridge_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{032a}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_inverted_double_arch_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{032b}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_caron_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{032c}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_circumflex_accent_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{032d}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_breve_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{032e}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_inverted_breve_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{032f}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_tilde_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0330}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_macron_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0331}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_low_line_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0332}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_double_low_line_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0333}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_tilde_overlay_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0334}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_short_stroke_overlay_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0335}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_long_stroke_overlay_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0336}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_short_solidus_overlay_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0337}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_long_solidus_overlay_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0338}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_right_half_ring_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0339}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_inverted_bridge_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{033a}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_square_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{033b}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_seagull_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{033c}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_x_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{033d}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_vertical_tilde_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{033e}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_double_overline_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{033f}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_grave_tone_mark_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0340}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_acute_tone_mark_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0341}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_greek_perispomeni_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0342}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_greek_koronis_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0343}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_greek_dialytika_tonos_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0344}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_greek_ypogegrammeni_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0345}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_bridge_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0346}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_equals_sign_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0347}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_double_vertical_line_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0348}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_left_angle_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0349}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_not_tilde_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{034a}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_homothetic_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{034b}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_almost_equal_to_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{034c}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_left_right_arrow_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{034d}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_upwards_arrow_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{034e}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_right_arrowhead_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0350}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_left_half_ring_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0351}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_fermata_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0352}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_x_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0353}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_left_arrowhead_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0354}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_right_arrowhead_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0355}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_right_half_ring_below_u0356_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0356}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_right_half_ring_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0357}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_light_centring_stroke_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0358}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_light_left_half_ring_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0359}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_almost_equal_to_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{035A}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_zigzag_above_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{035B}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_double_breve_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{035C}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_double_breve_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{035D}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_double_macron_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{035E}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_double_macron_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{035F}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_double_tilde_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0360}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_double_inverted_breve_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0361}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_double_rightwards_arrow_below_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0362}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_latin_small_letter_a_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0363}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_latin_small_letter_e_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0364}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_latin_small_letter_i_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0365}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_latin_small_letter_o_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0366}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_latin_small_letter_u_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0367}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_latin_small_letter_c_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0368}");
        assert!(args.parse_net_filter().is_none());
    }

    #[test]
    fn parse_net_filter_combining_latin_small_letter_d_after_prefix_returns_none() {
        let args = args_with_net_filter("10.0.0.0/24\u{0369}");
        assert!(args.parse_net_filter().is_none());
    }
}
