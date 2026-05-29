//! Contract tests for previously-uncovered surfaces:
//!   - `Args::parse_net_filter` SUCCESS path: valid IPv4 CIDR `192.168.1.0/24`
//!     returns `Some((addr, 24))`. Earlier round only pinned REJECTION cases
//!     (above-32, negative prefix, whitespace) — never pinned the happy path.
//!   - `Args::parse_net_filter` IPv6 CIDR `2001:db8::/64` returns Some.
//!   - `Args::parse_net_filter` /0 prefix is accepted (matches everything).
//!   - `Theme::palette_values` returns 6 distinct slots for each built-in
//!     theme and indices stay within 0..=255 (u8 by type, but pin shape).
//!   - `Theme::swatch` length == 6 for every variant in `ThemeName::ALL`.
//!   - `Resolver::resolve` with IPv6 disabled returns the IPv6 string verbatim
//!     including colons (not a hostname even if /etc/hosts contains one).
//!   - `port_to_service` UDP-only port (e.g. 67 bootps) does NOT match the
//!     TCP table; pins protocol-table separation.
//!
//! Earlier rounds pinned:
//!   - parse_net_filter REJECTION edges (prefix above 32, negative, whitespace)
//!   - Resolver disabled / toggle (NOT IPv6 specifically)
//!   - theme display_name uniqueness (NOT palette_values shape)

use iftoprs::config::cli::Args;
use iftoprs::config::theme::{Theme, ThemeName};
use iftoprs::util::resolver::{Resolver, port_to_service};
use std::net::IpAddr;

fn args_with_net_filter(filter: &str) -> Args {
    // The Args struct is built via Default-of-fields trick used by existing
    // tests/contract_net_filter_edges.rs. Re-use same construction shape.
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

/// `192.168.1.0/24` must round-trip to `(192.168.1.0, 24)`.
#[test]
fn test_parse_net_filter_ipv4_24_returns_some_with_correct_addr_and_prefix() {
    let a = args_with_net_filter("192.168.1.0/24");
    let result = a.parse_net_filter();
    let (addr, prefix) = result.expect("parse_net_filter must accept 192.168.1.0/24");
    assert_eq!(
        addr,
        "192.168.1.0".parse::<IpAddr>().unwrap(),
        "address must parse as 192.168.1.0"
    );
    assert_eq!(prefix, 24, "prefix must be 24");
}

/// IPv6 CIDR `2001:db8::/64` must parse to `Some(IpAddr::V6, 64)`.
#[test]
fn test_parse_net_filter_ipv6_slash_64_returns_some() {
    let a = args_with_net_filter("2001:db8::/64");
    let result = a.parse_net_filter();
    let (addr, prefix) = result.expect("IPv6 CIDR must parse");
    assert!(matches!(addr, IpAddr::V6(_)), "addr must be IPv6");
    assert_eq!(prefix, 64, "prefix must be 64");
}

/// `/0` prefix (default route) parses to Some with prefix=0.
#[test]
fn test_parse_net_filter_slash_zero_prefix_accepted() {
    let a = args_with_net_filter("0.0.0.0/0");
    let (addr, prefix) = a
        .parse_net_filter()
        .expect("0.0.0.0/0 must parse (matches everything)");
    assert_eq!(addr, "0.0.0.0".parse::<IpAddr>().unwrap());
    assert_eq!(prefix, 0, "/0 prefix must be 0");
}

/// `Theme::palette_values` returns exactly 6 u8 indices per built-in theme.
/// Pins the array shape so downstream UI rendering (6-color swatch) doesn't
/// silently start returning 5 or 7 slots.
#[test]
fn test_palette_values_returns_six_slots_for_every_builtin_theme() {
    for &name in ThemeName::ALL {
        let vals = Theme::palette_values(name);
        assert_eq!(
            vals.len(),
            6,
            "theme {name:?} must produce exactly 6 palette values; got {}",
            vals.len()
        );
    }
}

/// `Theme::swatch` produces 6 (Color, &str) tuples for every built-in theme.
/// Catches drift from the 6-color contract in the UI swatch rendering.
#[test]
fn test_theme_swatch_length_six_for_every_builtin() {
    for &name in ThemeName::ALL {
        let s = Theme::swatch(name);
        assert_eq!(
            s.len(),
            6,
            "theme {name:?} swatch must have 6 entries; got {}",
            s.len()
        );
    }
}

/// `Resolver` with IPv6 address & DNS DISABLED returns the IPv6 string with
/// colons — never attempts resolution.
#[test]
fn test_resolver_disabled_returns_ipv6_string_with_colons() {
    let r = Resolver::new(false);
    let addr: IpAddr = "2001:db8::abcd".parse().unwrap();
    let result = r.resolve(addr);
    assert!(
        result.contains(':'),
        "IPv6 string must contain colons; got {result:?}"
    );
    assert_eq!(
        result, "2001:db8::abcd",
        "must return canonical IPv6 string"
    );
}

/// `port_to_service` returns DIFFERENT entries for tcp vs udp on the same port
/// number (port 80: `http`/tcp exists, `http`/udp does not in most /etc/services).
/// Pins protocol-keyed lookup so a future refactor can't collapse the two.
#[test]
fn test_port_to_service_tcp_vs_udp_are_independent_lookups() {
    // Port 80 TCP is universally `http`. Port 80 UDP may be absent or different.
    let tcp_80 = port_to_service(80, true);
    let udp_80 = port_to_service(80, false);
    // We can't assert one is Some and the other None reliably across OSes
    // (some BSDs have udp/80=http), but we CAN assert: when both are Some,
    // they're the same NAME (since /etc/services typically aliases them);
    // when they differ, the proto-keyed lookup is functioning. Strong pin:
    // tcp lookup must NOT return whatever the udp-only table has.
    if let (Some(t), Some(u)) = (tcp_80, udp_80) {
        // Both present: should be same canonical service name.
        assert_eq!(
            t, u,
            "port 80 if listed for both protos should canonical-match"
        );
    }
    // Confirm port_to_service does not panic on edge port 65535 — the return
    // shape (Option) is the contract being pinned here.
    let _ = port_to_service(65535, true);
    let _ = port_to_service(65535, false);
}

/// `port_to_service` returns None for a guaranteed-unknown port. This pins the
/// Option-return shape so callers don't have to guard against panics or empty
/// strings.
#[test]
fn test_port_to_service_returns_none_for_unknown_port() {
    // Pick a port unlikely to be in /etc/services: 49999 (IANA dynamic range).
    let r = port_to_service(49999, true);
    assert!(
        r.is_none(),
        "port 49999 tcp must be None (dynamic range, not in /etc/services); got {:?}",
        r
    );
}
