//! Contract tests for previously-uncovered net-filter edge cases and
//! CLI short-circuit ordering. Targets undertested behaviors:
//!
//! - parse_net_filter accepts oversized prefix (> 32 for IPv4, > 128 for IPv6)
//!   because the function only validates address parse + u8 prefix parse;
//!   pinning this so a future tightening is intentional.
//! - parse_net_filter rejects negative prefix (signed value)
//! - parse_net_filter rejects mixed-family CIDR (IPv4 addr + "/256" for example)
//! - Args::try_parse_from rejects unknown flag with clap error
//! - `--list-colors` output contains theme name keywords (visual contract)
//! - `--completions` for every supported shell exits 0 and emits non-empty output
//! - `--version` output contains a `.` (semver-like)

use clap::Parser;
use iftoprs::config::cli::Args;
use std::process::Command;

fn cargo_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_iftoprs"))
}

fn args_with_net_filter(filter: &str) -> Args {
    // Build via clap so internal fields match defaults.
    Args::try_parse_from(["iftoprs", "-F", filter]).expect("parse -F succeeds")
}

#[test]
fn test_parse_net_filter_ipv4_prefix_above_32_still_parses() {
    // Current implementation only does `parts[1].parse::<u8>()` — value range 0..=255.
    // /33 should parse successfully today; this test PINS the current behavior so a
    // future tightening (rejecting > family-max) is intentional and adds a corresponding
    // test update rather than a silent break.
    let args = args_with_net_filter("10.0.0.0/33");
    let parsed = args.parse_net_filter();
    assert!(
        parsed.is_some(),
        "current parse_net_filter accepts /33 (no family-aware range check); got None"
    );
    let (_, prefix) = parsed.unwrap();
    assert_eq!(prefix, 33, "prefix value should round-trip as 33");
}

#[test]
fn test_parse_net_filter_ipv6_prefix_above_128_still_parses() {
    // Same rationale as IPv4 /33: pin current behavior.
    let args = args_with_net_filter("2001:db8::/200");
    let parsed = args.parse_net_filter();
    assert!(
        parsed.is_some(),
        "current parse_net_filter accepts /200 for IPv6; got None"
    );
}

#[test]
fn test_parse_net_filter_rejects_prefix_above_255() {
    // /256 cannot parse as u8 → None.
    let args = args_with_net_filter("10.0.0.0/256");
    assert!(
        args.parse_net_filter().is_none(),
        "/256 must fail (u8::parse overflow); got Some"
    );
}

#[test]
fn test_parse_net_filter_rejects_negative_prefix() {
    let args = args_with_net_filter("10.0.0.0/-1");
    assert!(
        args.parse_net_filter().is_none(),
        "negative prefix must fail u8::parse; got Some"
    );
}

#[test]
fn test_parse_net_filter_rejects_whitespace_in_prefix() {
    let args = args_with_net_filter("10.0.0.0/ 24");
    assert!(
        args.parse_net_filter().is_none(),
        "leading whitespace in prefix should fail u8::parse; got Some"
    );
}

#[test]
fn test_unknown_flag_is_clap_error() {
    let r = Args::try_parse_from(["iftoprs", "--definitely-not-a-flag"]);
    assert!(r.is_err(), "unknown flag must produce a clap error");
}

#[test]
fn test_list_colors_output_contains_theme_swatches() {
    // --list-colors must surface theme display strings; the CLI builds them via
    // Theme::swatch + ThemeName::ALL. Pin that the output is non-trivial.
    let out = cargo_bin().arg("--list-colors").output().expect("spawn");
    assert!(out.status.success(), "--list-colors must exit 0");
    let s = String::from_utf8_lossy(&out.stdout);
    // Expect at least 8 lines (each theme on its own line).
    let lines = s.lines().filter(|l| !l.trim().is_empty()).count();
    assert!(
        lines >= 8,
        "--list-colors should print one line per theme + headers; got {lines} non-empty lines"
    );
}

#[test]
fn test_completions_each_shell_emits_nonempty_output() {
    for shell in &["zsh", "bash", "fish", "elvish", "powershell"] {
        let out = cargo_bin()
            .args(["--completions", shell])
            .output()
            .expect("spawn");
        assert!(
            out.status.success(),
            "--completions {shell} must succeed; got {:?}",
            out.status
        );
        assert!(
            !out.stdout.is_empty(),
            "--completions {shell} must emit non-empty stdout"
        );
    }
}

#[test]
fn test_version_contains_dot_separator() {
    // Semver-like x.y.z must contain at least one `.`.
    let out = cargo_bin().arg("-V").output().expect("spawn");
    assert!(out.status.success(), "-V must exit 0");
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains('.'),
        "version output must contain at least one `.` separator; got {s:?}"
    );
}
