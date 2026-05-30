//! Round 4 contract tests for previously-uncovered surfaces:
//!   - `Args::no_processes` (`-Z`) is independent of `Args::hide_ports` (`-P`)
//!     — supplying one MUST NOT silently flip the other (catches a regression
//!     where short-flag aliasing or arg-table reordering could swap booleans)
//!   - `Args::json` is independent of `Args::list_colors` and
//!     `Args::list_interfaces` — three "modal" flags that must each parse to
//!     their own Args field, not a shared enum or single bool
//!   - `Args::parse_net_filter` returns None when input contains no `/` (no
//!     prefix) — pins the "must have CIDR slash" contract
//!   - `Args::parse_net_filter` returns None when input is the empty string
//!   - `Args::completions` Some/None bijection: omitted → None, supplied with
//!     a valid shell name → Some(matching Shell variant)
//!   - `Args::interface` / `Args::filter` / `Args::config` Option captures:
//!     each is `None` when the flag is omitted, `Some(value)` verbatim when
//!     supplied (no canonicalisation at the parse layer)
//!
//! Earlier rounds pinned:
//!   - parse_net_filter REJECTION edges (prefix above 32, negative, whitespace)
//!     and HAPPY path (192.168.1.0/24, ipv6, /0 prefix)
//!   - theme palette_values/swatch shape
//!   - Resolver disabled with IPv6
//!   - port_to_service tcp-vs-udp independence
//!
//! These tests pin DIFFERENT surfaces: boolean-flag independence, Option-field
//! bijection at the Args layer, parse_net_filter missing-slash / empty edges.

use clap::Parser;
use clap_complete::Shell;
use iftoprs::config::cli::Args;

/// `-Z` sets `no_processes=true` without touching `hide_ports`.
/// `-P` sets `hide_ports=true` without touching `no_processes`.
/// Catches a regression where the two short-flag handlers could collide.
#[test]
#[allow(non_snake_case)]
fn test_dash_Z_and_dash_P_are_independent_booleans() {
    let z_only = Args::parse_from(["iftoprs", "-Z"]);
    assert!(z_only.no_processes, "-Z must set no_processes");
    assert!(!z_only.hide_ports, "-Z alone must NOT set hide_ports");

    let p_only = Args::parse_from(["iftoprs", "-P"]);
    assert!(p_only.hide_ports, "-P must set hide_ports");
    assert!(!p_only.no_processes, "-P alone must NOT set no_processes");

    let both = Args::parse_from(["iftoprs", "-Z", "-P"]);
    assert!(both.no_processes && both.hide_ports, "-Z -P must set both");
}

/// `--json`, `--list-interfaces`, `--list-colors` are three independent
/// "modal" flags. Each one supplied alone must set ONLY its own bool field.
#[test]
fn test_json_list_interfaces_list_colors_are_three_independent_modal_flags() {
    let j = Args::parse_from(["iftoprs", "--json"]);
    assert!(j.json && !j.list_interfaces && !j.list_colors);

    let li = Args::parse_from(["iftoprs", "--list-interfaces"]);
    assert!(!li.json && li.list_interfaces && !li.list_colors);

    let lc = Args::parse_from(["iftoprs", "--list-colors"]);
    assert!(!lc.json && !lc.list_interfaces && lc.list_colors);
}

/// `parse_net_filter` returns None when input lacks a slash. The parser
/// requires CIDR notation — a bare host like "192.168.1.0" is rejected.
#[test]
fn test_parse_net_filter_returns_none_for_missing_slash() {
    let a = args_with_net_filter("192.168.1.0");
    assert!(
        a.parse_net_filter().is_none(),
        "input without slash must yield None (CIDR contract)"
    );
}

/// `parse_net_filter` returns None for the empty string.
#[test]
fn test_parse_net_filter_returns_none_for_empty_string() {
    let a = args_with_net_filter("");
    assert!(
        a.parse_net_filter().is_none(),
        "empty net-filter input must yield None"
    );
}

/// `Args::completions` Some/None bijection. Omitted → None. Supplied with
/// each clap-complete `Shell` name → Some(matching variant).
#[test]
fn test_completions_some_when_supplied_none_otherwise() {
    let none = Args::parse_from(["iftoprs"]);
    assert!(
        none.completions.is_none(),
        "completions must be None without --completions flag"
    );

    let bash = Args::parse_from(["iftoprs", "--completions", "bash"]);
    assert_eq!(
        bash.completions,
        Some(Shell::Bash),
        "--completions bash must yield Some(Shell::Bash)"
    );

    let zsh = Args::parse_from(["iftoprs", "--completions", "zsh"]);
    assert_eq!(
        zsh.completions,
        Some(Shell::Zsh),
        "--completions zsh must yield Some(Shell::Zsh)"
    );

    let fish = Args::parse_from(["iftoprs", "--completions", "fish"]);
    assert_eq!(
        fish.completions,
        Some(Shell::Fish),
        "--completions fish must yield Some(Shell::Fish)"
    );
}

/// `-i` / `-f` / `-c` Option fields capture their value verbatim — no
/// canonicalisation, no trimming, no path expansion at the Args layer.
#[test]
fn test_interface_filter_config_options_capture_value_verbatim() {
    let a = Args::parse_from([
        "iftoprs",
        "-i",
        "en0",
        "-f",
        "tcp port 80",
        "-c",
        "~/my.conf",
    ]);
    assert_eq!(a.interface.as_deref(), Some("en0"));
    assert_eq!(
        a.filter.as_deref(),
        Some("tcp port 80"),
        "BPF filter must be captured with embedded spaces, not split"
    );
    assert_eq!(
        a.config.as_deref(),
        Some("~/my.conf"),
        "config path is captured verbatim — no tilde expansion at parse layer"
    );

    let bare = Args::parse_from(["iftoprs"]);
    assert!(bare.interface.is_none(), "interface None when -i omitted");
    assert!(bare.filter.is_none(), "filter None when -f omitted");
    assert!(bare.config.is_none(), "config None when -c omitted");
}

// ─── Helper to construct an Args with only net_filter set ─────────────────

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
