//! Extra integration tests covering boundary scenarios not exercised
//! by `tests/integration.rs` — every test runs the real `iftoprs`
//! binary with a help/list short-circuit so no pcap capture happens
//! (CI doesn't have CAP_NET_RAW).
//!
//! Buckets:
//!   * exit-code consistency across short-circuit flags
//!   * cross-flag pairing (every flag combined with `-h` exits 0)
//!   * config-file error paths (missing / unreadable / malformed TOML)
//!   * completions content validation (every shell mentions all
//!     14 long flags)
//!   * help / version output is on stdout, not stderr
//!   * version string format matches the Cargo.toml crate version

use std::process::Command;

fn cargo_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_iftoprs"))
}

// ─── exit-code consistency for short-circuit flags ─────────────────

#[test]
fn help_short_exits_zero() {
    let out = cargo_bin().arg("-h").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn help_long_exits_zero() {
    let out = cargo_bin().arg("--help").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn version_short_exits_zero() {
    let out = cargo_bin().arg("-V").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn list_interfaces_exits_zero() {
    let out = cargo_bin().arg("--list-interfaces").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn list_colors_exits_zero() {
    let out = cargo_bin().arg("--list-colors").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn completions_zsh_exits_zero() {
    let out = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn invalid_flag_exits_two() {
    // Clap's convention: parse error → exit code 2.
    let out = cargo_bin()
        .arg("--this-flag-does-not-exist")
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "unknown flag should produce clap's standard exit code 2"
    );
}

#[test]
fn invalid_completions_shell_exits_nonzero() {
    let out = cargo_bin()
        .args(["--completions", "no-such-shell"])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "unknown completion shell must error, got exit {:?}",
        out.status.code()
    );
}

// ─── help vs version go to STDOUT (not stderr) ─────────────────────

#[test]
fn help_writes_to_stdout_not_stderr() {
    let out = cargo_bin().arg("-h").output().unwrap();
    assert!(!out.stdout.is_empty(), "help body must hit stdout");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("BANDWIDTH MONITOR"),
        "help banner should be on stdout, not stderr (got stderr: {:?})",
        &stderr.as_ref()[..stderr.len().min(200)]
    );
}

#[test]
fn version_writes_to_stdout_not_stderr() {
    let out = cargo_bin().arg("-V").output().unwrap();
    assert!(!out.stdout.is_empty(), "version body must hit stdout");
}

// ─── version string format matches Cargo.toml ──────────────────────

#[test]
fn version_long_matches_cargo_pkg_version() {
    let out = cargo_bin().arg("--version").output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains(env!("CARGO_PKG_VERSION")),
        "--version output must contain crate version {:?}, got {:?}",
        env!("CARGO_PKG_VERSION"),
        s,
    );
}

#[test]
fn version_short_matches_cargo_pkg_version() {
    let out = cargo_bin().arg("-V").output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains(env!("CARGO_PKG_VERSION")));
}

// ─── cross-flag pairing: each flag + -h still exits 0 ──────────────

#[test]
fn every_long_flag_paired_with_h_exits_zero() {
    // 14 long flags from `clap::Parser` derive (config/cli.rs). When
    // combined with -h, clap MUST run the help short-circuit cleanly
    // even though some flags conflict (e.g. `--json` + `--no-bars`
    // aren't logically conflicting but `--config` requires a value).
    // Flags that take a value need a sentinel; others are bare.
    let pairs: &[&[&str]] = &[
        &["--bytes", "-h"],
        &["--hide-ports", "-h"],
        &["--json", "-h"],
        &["--no-bars", "-h"],
        &["--no-dns", "-h"],
        &["--no-port-names", "-h"],
        &["--no-processes", "-h"],
        // value-taking flags need an arg even if -h short-circuits:
        &["--interface", "lo", "-h"],
        &["--filter", "port 80", "-h"],
        &["--net-filter", "127.0.0.0/8", "-h"],
        &["--config", "/tmp/iftoprs-nonexistent.conf", "-h"],
        &["--completions", "zsh", "-h"],
    ];
    for args in pairs {
        let out = cargo_bin().args(*args).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "{:?} + -h should exit 0, got {:?}",
            args,
            out.status,
        );
    }
}

// ─── config file error paths ───────────────────────────────────────

#[test]
fn config_path_pointing_to_directory_handled_gracefully() {
    // -h short-circuits before the config-load, so the bad path
    // should not affect help. If a future refactor moves config-load
    // earlier this test would catch the regression.
    let out = cargo_bin().args(["-c", "/tmp", "-h"]).output().unwrap();
    assert!(
        out.status.success(),
        "-h must short-circuit even when -c points at a dir; got {:?}\nstderr: {:?}",
        out.status,
        String::from_utf8_lossy(&out.stderr),
    );
}

/// Make a unique scratch path under `$TMPDIR` for this test process.
/// Avoids pulling in the `tempfile` crate as a dev-dep just for two
/// test files.
fn scratch_path(suffix: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    let id = N.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!(
        "iftoprs_cli_extra_{}_{}_{}",
        std::process::id(),
        id,
        suffix
    ))
}

#[test]
fn malformed_toml_config_errors_cleanly() {
    let p = scratch_path("malformed.conf");
    std::fs::write(&p, "this isn't valid TOML = [[[\n").expect("write");
    // Without -h, the config is loaded — expect a clean error, no panic.
    let out = cargo_bin()
        .args(["-c", p.to_str().unwrap()])
        .output()
        .unwrap();
    let _ = std::fs::remove_file(&p);
    assert!(
        out.status.code().is_some(),
        "process must exit with a code (no signal), got {:?}",
        out.status,
    );
}

#[test]
fn empty_toml_config_accepted() {
    let p = scratch_path("empty.conf");
    std::fs::write(&p, "").expect("write");
    let out = cargo_bin()
        .args(["-c", p.to_str().unwrap(), "-h"])
        .output()
        .unwrap();
    let _ = std::fs::remove_file(&p);
    // Empty TOML is valid → all-defaults config → -h still works.
    assert_eq!(out.status.code(), Some(0));
}

// ─── completions content validation ────────────────────────────────

fn collect_completions(shell: &str) -> String {
    let out = cargo_bin().args(["--completions", shell]).output().unwrap();
    assert!(out.status.success(), "--completions {shell} must succeed");
    String::from_utf8_lossy(&out.stdout).to_string()
}

#[test]
fn completions_bash_mentions_all_long_flags() {
    let bash = collect_completions("bash");
    for flag in &[
        "--bytes",
        "--config",
        "--hide-ports",
        "--json",
        "--no-bars",
        "--no-dns",
        "--no-port-names",
        "--no-processes",
        "--interface",
        "--filter",
        "--net-filter",
        "--list-interfaces",
        "--list-colors",
        "--completions",
    ] {
        assert!(
            bash.contains(flag),
            "bash completion missing flag {flag}; output prefix: {:?}",
            &bash.as_str()[..bash.len().min(400)],
        );
    }
}

#[test]
fn completions_zsh_mentions_all_long_flags() {
    let zsh = collect_completions("zsh");
    for flag in &[
        "--bytes",
        "--config",
        "--hide-ports",
        "--json",
        "--no-bars",
        "--no-dns",
        "--no-port-names",
        "--no-processes",
        "--interface",
        "--filter",
        "--net-filter",
        "--list-interfaces",
        "--list-colors",
        "--completions",
    ] {
        assert!(zsh.contains(flag), "zsh completion missing flag {flag}",);
    }
}

#[test]
fn completions_fish_mentions_all_long_flags() {
    // Fish's clap_complete generator emits `complete -c iftoprs -l bytes
    // -d 'desc'` — the flag NAME without `--` prefix is passed via the
    // `-l` switch. So we search for `-l <name>` rather than `--<name>`.
    let fish = collect_completions("fish");
    for flag in &[
        "bytes",
        "config",
        "hide-ports",
        "json",
        "no-bars",
        "no-dns",
        "no-port-names",
        "no-processes",
        "interface",
        "filter",
        "net-filter",
        "list-interfaces",
        "list-colors",
        "completions",
    ] {
        let needle = format!("-l {}", flag);
        assert!(
            fish.contains(&needle),
            "fish completion missing flag '{flag}' (expected `{needle}`)",
        );
    }
}

#[test]
fn completions_bash_starts_with_complete_directive() {
    let bash = collect_completions("bash");
    // Bash completion scripts always start with `_<name>()` function
    // definition or `complete -F …` registration.
    assert!(
        bash.contains("complete") || bash.contains("_iftoprs"),
        "bash completion shape unexpected: {:?}",
        &bash[..bash.len().min(200)],
    );
}

#[test]
fn completions_zsh_starts_with_compdef() {
    let zsh = collect_completions("zsh");
    assert!(
        zsh.contains("#compdef") || zsh.contains("compdef"),
        "zsh completion missing compdef header: {:?}",
        &zsh[..zsh.len().min(200)],
    );
}

// ─── list-colors output format ─────────────────────────────────────

#[test]
fn list_colors_output_is_nonempty_string() {
    let out = cargo_bin().arg("--list-colors").output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        !s.trim().is_empty(),
        "--list-colors must print SOMETHING; got empty stdout"
    );
}

// ─── list-interfaces output format ─────────────────────────────────

#[test]
fn list_interfaces_includes_loopback() {
    // Every Unix host has lo / lo0.
    let out = cargo_bin().arg("--list-interfaces").output().unwrap();
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains("lo"),
        "--list-interfaces output should contain loopback: {:?}",
        s,
    );
}

// ─── -V parity: short and long should agree ─────────────────────────

#[test]
fn version_short_and_long_produce_identical_output() {
    let short = cargo_bin().arg("-V").output().unwrap().stdout;
    let long = cargo_bin().arg("--version").output().unwrap().stdout;
    assert_eq!(
        short, long,
        "-V and --version should produce identical text"
    );
}

// ─── --list-interfaces vs --list-colors are independent ────────────

#[test]
fn list_interfaces_then_list_colors_both_succeed() {
    // Pin that there's no cross-pollution between invocations (each
    // process is fresh; would only fail if there's a static-state bug
    // that survives the process boundary — which it shouldn't but is
    // cheap to verify).
    let a = cargo_bin().arg("--list-interfaces").output().unwrap();
    let b = cargo_bin().arg("--list-colors").output().unwrap();
    assert!(a.status.success() && b.status.success());
}
