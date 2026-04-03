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
    }

    #[test]
    fn clap_parse_list_colors() {
        let args = Args::try_parse_from(["iftoprs", "--list-colors"]).unwrap();
        assert!(args.list_colors);
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
}
