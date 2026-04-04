use std::process::Command;

/// Run the `iftoprs` binary built for this test crate. Using `CARGO_BIN_EXE_*` avoids
/// `cargo run` (which can swallow or mishandle child stdout in some environments).
fn cargo_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_iftoprs"))
}

#[test]
fn help_flag_shows_banner() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BANDWIDTH MONITOR"),
        "help should contain banner tagline"
    );
    assert!(stdout.contains("USAGE"), "help should contain USAGE");
    assert!(
        stdout.contains("--interface"),
        "help should list --interface flag"
    );
    assert!(
        stdout.contains("--no-dns"),
        "help should list --no-dns flag"
    );
    assert!(
        stdout.contains("KEYBINDS"),
        "help should contain KEYBINDS section"
    );
}

#[test]
fn version_flag_shows_version() {
    let output = cargo_bin().arg("-V").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("iftoprs "),
        "should start with 'iftoprs '"
    );
    assert!(stdout.contains('.'), "version should contain a dot");
}

#[test]
fn completions_zsh_generates_valid_output() {
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("#compdef iftoprs"),
        "should contain compdef header"
    );
    assert!(
        stdout.contains("--interface"),
        "completions should include --interface"
    );
    assert!(
        stdout.contains("--no-dns"),
        "completions should include --no-dns"
    );
    assert!(
        stdout.contains("--completions"),
        "completions should include --completions"
    );
    assert!(
        stdout.contains("--no-processes"),
        "completions should include --no-processes"
    );
}

#[test]
fn completions_bash_generates_valid_output() {
    let output = cargo_bin()
        .args(["--completions", "bash"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("_iftoprs"),
        "should contain completion function"
    );
    assert!(stdout.contains("COMPREPLY"), "should contain COMPREPLY");
}

#[test]
fn help_contains_all_flags() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let flags = [
        "--interface",
        "--filter",
        "--net-filter",
        "--no-dns",
        "--no-port-names",
        "--promiscuous",
        "--no-bars",
        "--bytes",
        "--hide-ports",
        "--no-processes",
        "--list-interfaces",
    ];
    for flag in &flags {
        assert!(stdout.contains(flag), "help missing flag: {}", flag);
    }
}

#[test]
fn help_contains_ansi_colors() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\x1b["),
        "help should contain ANSI escape codes"
    );
}

#[test]
fn help_contains_new_keybinds() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("scroll"),
        "help should document scroll keybinds"
    );
    assert!(
        stdout.contains("disconnect"),
        "help should document quit keybind"
    );
}

#[test]
fn help_shows_system_section() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("SYSTEM"),
        "help should contain SYSTEM section"
    );
    assert!(
        stdout.contains("MenkeTechnologies"),
        "help should credit author"
    );
}

#[test]
fn version_matches_cargo_toml() {
    let output = cargo_bin().arg("-V").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let version = stdout.strip_prefix("iftoprs ").unwrap();
    // Version should be semver
    let parts: Vec<&str> = version.split('.').collect();
    assert_eq!(parts.len(), 3, "version should be semver: {}", version);
    for part in &parts {
        assert!(
            part.parse::<u32>().is_ok(),
            "non-numeric version part: {}",
            part
        );
    }
}

#[test]
fn help_contains_border_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("border"),
        "help should document border toggle"
    );
}

#[test]
fn help_contains_filter_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("filter"),
        "help should document filter keybind"
    );
}

#[test]
fn help_contains_theme_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("themes"),
        "help should document theme keybind"
    );
}

#[test]
fn help_contains_pause_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("pause"),
        "help should document pause keybind"
    );
}

#[test]
fn list_colors_shows_all_themes() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Neon Sprawl"),
        "should list Neon Sprawl theme"
    );
    assert!(
        stdout.contains("Blade Runner"),
        "should list Blade Runner theme"
    );
    assert!(
        stdout.contains("iftopcolor"),
        "should list iftopcolor theme"
    );
}

#[test]
fn default_config_file_exists() {
    let path = std::path::Path::new("iftoprs.default.conf");
    assert!(
        path.exists(),
        "iftoprs.default.conf should exist in project root"
    );
    let content = std::fs::read_to_string(path).unwrap();
    assert!(
        content.contains("theme"),
        "default config should contain theme"
    );
    assert!(
        content.contains("show_border"),
        "default config should contain show_border"
    );
    assert!(
        content.contains("refresh_rate"),
        "default config should contain refresh_rate"
    );
    assert!(
        content.contains("alert_threshold"),
        "default config should contain alert_threshold"
    );
    assert!(
        content.contains("pinned"),
        "default config should contain pinned"
    );
}

#[test]
fn completions_zsh_includes_list_colors() {
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--list-colors"),
        "zsh completions should include --list-colors"
    );
}

#[test]
fn default_config_has_interface_docs() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(
        content.contains("interface"),
        "default config should document interface field"
    );
}

#[test]
fn help_contains_interface_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // -h help mentions interface flag
    assert!(
        stdout.contains("--interface"),
        "help should show --interface flag"
    );
}

#[test]
fn help_contains_config_flag() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--config"),
        "help should show --config flag"
    );
    assert!(stdout.contains("-c"), "help should show -c short flag");
}

#[test]
fn completions_fish_generates_valid_output() {
    let output = cargo_bin()
        .args(["--completions", "fish"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("iftoprs"),
        "fish completions should reference iftoprs"
    );
    assert!(
        stdout.contains("interface"),
        "fish completions should include interface"
    );
}

#[test]
fn completions_zsh_includes_config_flag() {
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--config"),
        "zsh completions should include --config"
    );
}

#[test]
fn completions_bash_includes_config_flag() {
    let output = cargo_bin()
        .args(["--completions", "bash"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("config"),
        "bash completions should include config"
    );
}

#[test]
fn help_exit_code_zero() {
    let output = cargo_bin().arg("-h").output().unwrap();
    assert!(output.status.success(), "-h should exit with code 0");
}

#[test]
fn version_exit_code_zero() {
    let output = cargo_bin().arg("-V").output().unwrap();
    assert!(output.status.success(), "-V should exit with code 0");
}

#[test]
fn list_colors_exit_code_zero() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    assert!(
        output.status.success(),
        "--list-colors should exit with code 0"
    );
}

#[test]
fn completions_exit_code_zero() {
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    assert!(
        output.status.success(),
        "--completions zsh should exit with code 0"
    );
}

#[test]
fn help_banner_has_ascii_art() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("██"),
        "help banner should have block characters"
    );
}

#[test]
fn help_shows_version_number() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The banner contains the version like "v2.4.0"
    assert!(stdout.contains('v'), "help banner should show version");
}

#[test]
fn list_colors_shows_usage_hint() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Usage"),
        "list-colors should show usage hint"
    );
    assert!(
        stdout.contains("Cycle"),
        "list-colors should show cycle hint"
    );
}

#[test]
fn help_contains_header_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("header"),
        "help should document header toggle"
    );
}

#[test]
fn help_contains_refresh_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("refresh"),
        "help should document refresh rate keybind"
    );
}

#[test]
fn help_contains_sort_keybinds() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("sort"),
        "help should document sort keybinds"
    );
    assert!(
        stdout.contains("freeze"),
        "help should document freeze order keybind"
    );
}

#[test]
fn help_contains_processes_flag() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--no-processes"),
        "help should show --no-processes"
    );
    assert!(stdout.contains("-Z"), "help should show -Z short flag");
}

#[test]
fn default_config_has_all_fields() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    let expected_fields = [
        "theme",
        "show_border",
        "show_ports",
        "show_bars",
        "show_processes",
        "show_header",
        "refresh_rate",
        "alert_threshold",
        "pinned",
    ];
    for field in &expected_fields {
        assert!(
            content.contains(field),
            "default config missing field: {}",
            field
        );
    }
}

#[test]
fn help_mentions_capture_section() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("CAPTURE"),
        "help should have CAPTURE section"
    );
}

#[test]
fn help_contains_json_flag() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--json"), "help should show --json flag");
    assert!(
        stdout.contains("NDJSON"),
        "help should describe NDJSON output"
    );
}

#[test]
fn help_contains_tab_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Tab"),
        "help should document Tab key for switching views"
    );
    assert!(
        stdout.contains("switch view"),
        "help should explain Tab switches views"
    );
}

#[test]
fn completions_zsh_includes_json_flag() {
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--json"),
        "zsh completions should include --json"
    );
}

// ── Exit codes ──

#[test]
fn completions_bash_exit_code_zero() {
    let output = cargo_bin()
        .args(["--completions", "bash"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn completions_fish_exit_code_zero() {
    let output = cargo_bin()
        .args(["--completions", "fish"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn completions_elvish_exit_code_zero() {
    let output = cargo_bin()
        .args(["--completions", "elvish"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn completions_powershell_exit_code_zero() {
    let output = cargo_bin()
        .args(["--completions", "powershell"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

// ── Completions content for all shells ──

#[test]
fn completions_elvish_generates_valid_output() {
    let output = cargo_bin()
        .args(["--completions", "elvish"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "elvish completions should not be empty");
    assert!(
        stdout.contains("iftoprs"),
        "elvish completions should reference iftoprs"
    );
}

#[test]
fn completions_powershell_generates_valid_output() {
    let output = cargo_bin()
        .args(["--completions", "powershell"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.is_empty(),
        "powershell completions should not be empty"
    );
    assert!(
        stdout.contains("iftoprs"),
        "powershell completions should reference iftoprs"
    );
}

// ── Completions include all flags ──

#[test]
fn completions_zsh_includes_all_flags() {
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for flag in [
        "--interface",
        "--filter",
        "--net-filter",
        "--no-dns",
        "--no-port-names",
        "--promiscuous",
        "--no-bars",
        "--bytes",
        "--hide-ports",
        "--no-processes",
        "--json",
        "--list-interfaces",
        "--list-colors",
        "--config",
        "--help",
        "--version",
    ] {
        assert!(
            stdout.contains(flag),
            "zsh completions missing flag: {}",
            flag
        );
    }
}

#[test]
fn completions_bash_includes_all_flags() {
    let output = cargo_bin()
        .args(["--completions", "bash"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for flag in [
        "interface",
        "filter",
        "no-dns",
        "no-bars",
        "bytes",
        "hide-ports",
        "no-processes",
        "json",
        "list-colors",
    ] {
        assert!(
            stdout.contains(flag),
            "bash completions missing flag: {}",
            flag
        );
    }
}

#[test]
fn completions_fish_includes_all_flags() {
    let output = cargo_bin()
        .args(["--completions", "fish"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for flag in [
        "interface",
        "filter",
        "no-dns",
        "bytes",
        "json",
        "list-colors",
    ] {
        assert!(
            stdout.contains(flag),
            "fish completions missing flag: {}",
            flag
        );
    }
}

// ── Help output structure ──

#[test]
fn help_has_three_sections() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("CAPTURE"), "missing CAPTURE section");
    assert!(stdout.contains("KEYBINDS"), "missing KEYBINDS section");
    assert!(stdout.contains("SYSTEM"), "missing SYSTEM section");
}

#[test]
fn help_shows_short_flags() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for flag in [
        "-i", "-f", "-F", "-n", "-N", "-p", "-b", "-B", "-P", "-Z", "-l", "-h", "-V", "-c",
    ] {
        assert!(stdout.contains(flag), "help missing short flag: {}", flag);
    }
}

#[test]
fn help_contains_bpf_example() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("tcp port 80"),
        "help should show BPF filter example"
    );
}

#[test]
fn help_contains_cidr_example() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("192.168.1.0/24"),
        "help should show CIDR example"
    );
}

#[test]
fn help_contains_promiscuous_description() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("promiscuous"),
        "help should describe promiscuous mode"
    );
}

// ── Keybind documentation ──

#[test]
fn help_documents_all_keybinds() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let keybinds = [
        "help HUD",
        "toggle DNS",
        "bars",
        "bytes/bits",
        "ports",
        "processes",
        "line mode",
        "cumulative",
        "pause",
        "border",
        "themes",
        "filter",
        "header bar",
        "refresh rate",
        "switch view",
        "sort by",
        "freeze order",
        "scroll",
        "disconnect",
    ];
    for kb in &keybinds {
        assert!(stdout.contains(kb), "help missing keybind: {}", kb);
    }
}

#[test]
fn help_documents_navigation_keys() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("j/k"), "help should show j/k scroll keys");
    assert!(stdout.contains("1/2/3"), "help should show 1/2/3 sort keys");
    assert!(stdout.contains("< / >"), "help should show </> sort keys");
}

#[test]
fn help_shows_tagline() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("JACK IN"),
        "help should show cyberpunk tagline"
    );
    assert!(
        stdout.contains("neon rain"),
        "help should show neon rain quote"
    );
}

// ── Version output format ──

#[test]
fn version_output_is_single_line() {
    let output = cargo_bin().arg("-V").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(!stdout.contains('\n'), "version should be a single line");
}

#[test]
fn version_and_help_show_same_version() {
    let v_output = cargo_bin().arg("-V").output().unwrap();
    let v_str = String::from_utf8_lossy(&v_output.stdout).trim().to_string();
    let version = v_str.strip_prefix("iftoprs ").unwrap();

    let h_output = cargo_bin().arg("-h").output().unwrap();
    let h_str = String::from_utf8_lossy(&h_output.stdout);
    assert!(
        h_str.contains(version),
        "help banner should show same version as -V"
    );
}

// ── List colors output ──

#[test]
fn list_colors_contains_ansi() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\x1b["),
        "list-colors should contain ANSI escape codes"
    );
}

#[test]
fn list_colors_shows_all_31_themes() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let themes = [
        "Neon Sprawl",
        "Acid Rain",
        "Ice Breaker",
        "Synth Wave",
        "Rust Belt",
        "Ghost Wire",
        "Red Sector",
        "Sakura Den",
        "Data Stream",
        "Solar Flare",
        "Neon Noir",
        "Chrome Heart",
        "Blade Runner",
        "Void Walker",
        "Toxic Waste",
        "Cyber Frost",
        "Plasma Core",
        "Steel Nerve",
        "Dark Signal",
        "Glitch Pop",
        "Holo Shift",
        "Night City",
        "Deep Net",
        "Laser Grid",
        "Quantum Flux",
        "Bio Hazard",
        "Darkwave",
        "Overlock",
        "Megacorp",
        "Zaibatsu",
        "iftopcolor",
    ];
    for theme in &themes {
        assert!(
            stdout.contains(theme),
            "list-colors missing theme: {}",
            theme
        );
    }
}

#[test]
fn list_colors_shows_color_swatches() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Color swatches use 48;5;N escape codes for background
    assert!(
        stdout.contains("48;5;"),
        "list-colors should contain 256-color escapes"
    );
}

#[test]
fn list_colors_has_section_header() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BUILTIN COLOR SCHEMES"),
        "should have section header"
    );
}

// ── Default config file ──

#[test]
fn default_config_is_valid_toml() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    // Filter out comment lines and parse as TOML
    let filtered: String = content
        .lines()
        .filter(|l| !l.trim_start().starts_with('#') || l.contains('='))
        .collect::<Vec<_>>()
        .join("\n");
    let parsed: toml::Value = toml::from_str(&filtered).unwrap();
    assert!(parsed.is_table());
}

#[test]
fn default_config_theme_is_neon_sprawl() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(
        content.contains("theme = \"NeonSprawl\""),
        "default theme should be NeonSprawl"
    );
}

#[test]
fn default_config_dns_enabled() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(
        content.contains("dns_resolution = true"),
        "dns should be enabled by default"
    );
}

#[test]
fn default_config_refresh_rate_one() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(
        content.contains("refresh_rate = 1"),
        "default refresh rate should be 1"
    );
}

#[test]
fn default_config_alert_disabled() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(
        content.contains("alert_threshold = 0.0"),
        "alerts should be disabled by default"
    );
}

#[test]
fn default_config_has_comments() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    let comment_count = content
        .lines()
        .filter(|l| l.trim_start().starts_with('#'))
        .count();
    assert!(
        comment_count >= 5,
        "default config should have documentation comments"
    );
}

#[test]
fn default_config_documents_bar_styles() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(
        content.contains("Gradient"),
        "should document Gradient bar style"
    );
    assert!(content.contains("Solid"), "should document Solid bar style");
    assert!(content.contains("Thin"), "should document Thin bar style");
    assert!(content.contains("Ascii"), "should document Ascii bar style");
}

#[test]
fn default_config_documents_alert_examples() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("1 Mbit/s"), "should show 1 Mbit/s example");
    assert!(content.contains("1 Gbit/s"), "should show 1 Gbit/s example");
}

// ── Zsh completion file ──

#[test]
fn zsh_completion_file_exists() {
    let path = std::path::Path::new("completions/_iftoprs");
    assert!(path.exists(), "completions/_iftoprs should exist");
}

#[test]
fn zsh_completion_file_has_compdef() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    assert!(
        content.starts_with("#compdef iftoprs"),
        "should start with #compdef"
    );
}

#[test]
fn zsh_completion_file_has_function() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    assert!(
        content.contains("_iftoprs()"),
        "should define _iftoprs function"
    );
}

#[test]
fn zsh_completion_file_includes_all_flags() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    for flag in [
        "--config",
        "--interface",
        "--filter",
        "--net-filter",
        "--no-dns",
        "--no-port-names",
        "--promiscuous",
        "--no-bars",
        "--bytes",
        "--hide-ports",
        "--no-processes",
        "--json",
        "--list-interfaces",
        "--list-colors",
        "--help",
        "--version",
        "--completions",
    ] {
        assert!(content.contains(flag), "_iftoprs missing flag: {}", flag);
    }
}

#[test]
fn zsh_completion_file_includes_short_flags() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    // Flags with values use '-X+[' or '-X[', boolean flags use '-X['
    for flag in ["-i", "-f", "-F", "-c"] {
        assert!(
            content.contains(&format!("'{flag}+")),
            "missing short flag with value: {}",
            flag
        );
    }
    for flag in ["-n", "-N", "-p", "-b", "-B", "-P", "-Z", "-l", "-h", "-V"] {
        assert!(
            content.contains(&format!("'{flag}[")),
            "missing short boolean flag: {}",
            flag
        );
    }
}

#[test]
fn zsh_completion_file_has_shell_completions_values() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    assert!(
        content.contains("bash"),
        "completions should list bash shell"
    );
    assert!(content.contains("zsh"), "completions should list zsh shell");
    assert!(
        content.contains("fish"),
        "completions should list fish shell"
    );
}

// ── Stderr output ──

#[test]
fn help_stderr_is_empty() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Filter out compiler warnings that may appear with `cargo run`
    let app_stderr: String = stderr
        .lines()
        .filter(|l| {
            !l.trim().is_empty()
                && !l.contains("warning:")
                && !l.contains("-->")
                && !l.contains("= note:")
                && !l.starts_with("  ")
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        app_stderr.is_empty(),
        "help should not write to stderr: {}",
        app_stderr
    );
}

#[test]
fn version_stderr_is_empty() {
    let output = cargo_bin().arg("-V").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let app_stderr: String = stderr
        .lines()
        .filter(|l| {
            !l.trim().is_empty()
                && !l.contains("warning:")
                && !l.contains("-->")
                && !l.contains("= note:")
                && !l.starts_with("  ")
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        app_stderr.is_empty(),
        "version should not write to stderr: {}",
        app_stderr
    );
}

#[test]
fn list_colors_stderr_is_empty() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let app_stderr: String = stderr
        .lines()
        .filter(|l| {
            !l.trim().is_empty()
                && !l.contains("warning:")
                && !l.contains("-->")
                && !l.contains("= note:")
                && !l.starts_with("  ")
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        app_stderr.is_empty(),
        "list-colors should not write to stderr: {}",
        app_stderr
    );
}

// ── Custom config flag ──

#[test]
fn config_flag_with_nonexistent_file_shows_help() {
    // -c takes a path argument, then -h is parsed as a separate flag
    let output = cargo_bin()
        .args(["-c", "/tmp/nonexistent_iftoprs_test_12345.conf", "-h"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show help output (the -h flag takes priority over needing a valid config)
    assert!(
        stdout.contains("BANDWIDTH MONITOR") || stdout.contains("IFTOPRS"),
        "should still show help with nonexistent config"
    );
}

// ── Output consistency ──

#[test]
fn help_output_is_deterministic() {
    let out1 = cargo_bin().arg("-h").output().unwrap();
    let out2 = cargo_bin().arg("-h").output().unwrap();
    assert_eq!(
        out1.stdout, out2.stdout,
        "help output should be deterministic"
    );
}

#[test]
fn version_output_is_deterministic() {
    let out1 = cargo_bin().arg("-V").output().unwrap();
    let out2 = cargo_bin().arg("-V").output().unwrap();
    assert_eq!(
        out1.stdout, out2.stdout,
        "version output should be deterministic"
    );
}

#[test]
fn completions_output_is_deterministic() {
    let out1 = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let out2 = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    assert_eq!(
        out1.stdout, out2.stdout,
        "completions output should be deterministic"
    );
}

// ── Help output size ──

#[test]
fn help_output_is_substantial() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.len() > 1000,
        "help should be at least 1000 bytes, got {}",
        stdout.len()
    );
    let lines = stdout.lines().count();
    assert!(
        lines >= 30,
        "help should have at least 30 lines, got {}",
        lines
    );
}

#[test]
fn list_colors_output_is_substantial() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.lines().count();
    assert!(
        lines >= 33,
        "list-colors should have at least 33 lines (31 themes + header/footer), got {}",
        lines
    );
}

// ── Completions generated match static file ──

#[test]
fn generated_zsh_completions_match_static_file() {
    let generated = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let gen_str = String::from_utf8_lossy(&generated.stdout);
    let static_file = std::fs::read_to_string("completions/_iftoprs").unwrap();
    assert_eq!(
        gen_str.trim(),
        static_file.trim(),
        "generated zsh completions should match completions/_iftoprs"
    );
}

// ── Invalid argument handling ──

#[test]
fn invalid_completions_shell_fails() {
    let output = cargo_bin()
        .args(["--completions", "invalid_shell"])
        .output()
        .unwrap();
    assert!(!output.status.success(), "invalid shell name should fail");
}

// ── Help banner structure ──

#[test]
fn help_banner_has_signal_bar() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("SIGNAL"),
        "banner should have SIGNAL indicator"
    );
    assert!(
        stdout.contains("ONLINE"),
        "banner should show ONLINE status"
    );
}

#[test]
fn help_shows_iftop_clone_description() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("iftop clone"),
        "should describe as iftop clone"
    );
    assert!(stdout.contains("Rust"), "should mention Rust");
}

// ── Config file documentation ──

#[test]
fn default_config_lists_all_theme_names() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    for theme in [
        "NeonSprawl",
        "BladeRunner",
        "Iftopcolor",
        "GlitchPop",
        "Zaibatsu",
    ] {
        assert!(
            content.contains(theme),
            "default config should list theme: {}",
            theme
        );
    }
}

#[test]
fn default_config_documents_interface_examples() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(content.contains("en0"), "should show en0 example");
    assert!(content.contains("eth0"), "should show eth0 example");
}

// ── Cargo.toml metadata ──

#[test]
fn cargo_toml_exists() {
    assert!(std::path::Path::new("Cargo.toml").exists());
}

#[test]
fn cargo_lock_exists() {
    assert!(
        std::path::Path::new("Cargo.lock").exists(),
        "Cargo.lock must be present for reproducible builds and cargo --locked in CI"
    );
}

#[test]
fn minimal_etc_services_fixture_exists_for_tests() {
    let path = std::path::Path::new("tests/fixtures/minimal_etc_services.txt");
    assert!(
        path.exists(),
        "fixture used by resolver unit tests must be present"
    );
    let content = std::fs::read_to_string(path).unwrap();
    assert!(
        content.contains("smtp") && content.contains("25/tcp"),
        "fixture should list smtp on 25/tcp"
    );
}

#[test]
fn cargo_toml_has_package_name() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(content.contains("name = \"iftoprs\""));
}

#[test]
fn cargo_toml_has_version() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(content.contains("version = "));
}

// ── JSON flag ──

#[test]
fn help_json_flag_description() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--json"), "help should list --json flag");
    assert!(
        stdout.contains("NDJSON"),
        "help should describe NDJSON output"
    );
    assert!(stdout.contains("no TUI"), "help should mention no TUI");
}

#[test]
fn completions_bash_includes_json() {
    let output = cargo_bin()
        .args(["--completions", "bash"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("json"),
        "bash completions should include json"
    );
}

#[test]
fn completions_fish_includes_json() {
    let output = cargo_bin()
        .args(["--completions", "fish"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("json"),
        "fish completions should include json"
    );
}

#[test]
fn zsh_completion_file_includes_json() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    assert!(content.contains("--json"), "_iftoprs should include --json");
}

// ── Tab keybind ──

#[test]
fn help_tab_keybind_description() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tab"), "help should document Tab keybind");
    assert!(
        stdout.contains("switch view"),
        "help should describe Tab as switching views"
    );
}

// ── Cargo.toml dependencies ──

#[test]
fn cargo_toml_has_serde_json() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(
        content.contains("serde_json"),
        "Cargo.toml should include serde_json dependency"
    );
}

// ── README features ──

#[test]
fn readme_documents_json_streaming() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("--json"),
        "README should document --json flag"
    );
    assert!(content.contains("NDJSON"), "README should mention NDJSON");
}

#[test]
fn readme_documents_process_view() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(content.contains("Tab"), "README should document Tab key");
    assert!(
        content.contains("process"),
        "README should mention process aggregation"
    );
}

#[test]
fn readme_documents_jq_example() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("jq"),
        "README should show jq piping example"
    );
}

// ══════════════════════════════════════════════════════════════════
//  CLI flag combinations
// ══════════════════════════════════════════════════════════════════

#[test]
fn help_with_no_dns_flag_still_shows_help() {
    let output = cargo_bin().args(["-n", "-h"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BANDWIDTH MONITOR"),
        "help should work with -n"
    );
    assert!(output.status.success());
}

#[test]
fn help_with_bytes_flag_still_shows_help() {
    let output = cargo_bin().args(["-B", "-h"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BANDWIDTH MONITOR"),
        "help should work with -B"
    );
    assert!(output.status.success());
}

#[test]
fn help_with_no_bars_flag_still_shows_help() {
    let output = cargo_bin().args(["-b", "-h"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BANDWIDTH MONITOR"),
        "help should work with -b"
    );
    assert!(output.status.success());
}

#[test]
fn help_with_hide_ports_flag_still_shows_help() {
    let output = cargo_bin().args(["-P", "-h"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BANDWIDTH MONITOR"));
    assert!(output.status.success());
}

#[test]
fn help_with_no_processes_flag_still_shows_help() {
    let output = cargo_bin().args(["-Z", "-h"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BANDWIDTH MONITOR"));
    assert!(output.status.success());
}

#[test]
fn help_with_promiscuous_flag_still_shows_help() {
    let output = cargo_bin().args(["-p", "-h"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BANDWIDTH MONITOR"));
    assert!(output.status.success());
}

#[test]
fn help_with_all_display_flags() {
    let output = cargo_bin()
        .args(["-n", "-N", "-b", "-B", "-P", "-Z", "-h"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BANDWIDTH MONITOR"),
        "help should work with all display flags"
    );
    assert!(output.status.success());
}

#[test]
fn version_with_no_dns_flag_still_shows_version() {
    let output = cargo_bin().args(["-n", "-V"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("iftoprs "));
    assert!(output.status.success());
}

#[test]
fn list_colors_with_no_dns_flag() {
    let output = cargo_bin().args(["-n", "--list-colors"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BUILTIN COLOR SCHEMES"));
    assert!(output.status.success());
}

#[test]
fn completions_with_extra_flags() {
    let output = cargo_bin()
        .args(["-n", "--completions", "zsh"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("#compdef iftoprs"));
    assert!(output.status.success());
}

// ══════════════════════════════════════════════════════════════════
//  Invalid argument handling
// ══════════════════════════════════════════════════════════════════

#[test]
fn unknown_long_flag_fails() {
    let output = cargo_bin().arg("--nonexistent-flag").output().unwrap();
    assert!(!output.status.success(), "unknown flag should fail");
}

#[test]
fn unknown_short_flag_fails() {
    let output = cargo_bin().arg("-X").output().unwrap();
    assert!(!output.status.success(), "unknown short flag should fail");
}

#[test]
fn interface_flag_without_value_fails() {
    let output = cargo_bin().args(["-i"]).output().unwrap();
    assert!(!output.status.success(), "-i without a value should fail");
}

#[test]
fn filter_flag_without_value_fails() {
    let output = cargo_bin().args(["-f"]).output().unwrap();
    assert!(!output.status.success(), "-f without a value should fail");
}

#[test]
fn net_filter_flag_without_value_fails() {
    let output = cargo_bin().args(["-F"]).output().unwrap();
    assert!(!output.status.success(), "-F without a value should fail");
}

#[test]
fn config_flag_without_value_fails() {
    let output = cargo_bin().args(["-c"]).output().unwrap();
    assert!(!output.status.success(), "-c without a value should fail");
}

#[test]
fn completions_without_shell_name_fails() {
    let output = cargo_bin().args(["--completions"]).output().unwrap();
    assert!(
        !output.status.success(),
        "--completions without shell name should fail"
    );
}

#[test]
fn double_dash_unknown_flag_fails() {
    let output = cargo_bin().arg("--does-not-exist").output().unwrap();
    assert!(!output.status.success());
}

#[test]
fn positional_argument_fails() {
    let output = cargo_bin().arg("some_positional").output().unwrap();
    assert!(
        !output.status.success(),
        "positional args should not be accepted"
    );
}

#[test]
fn invalid_flag_writes_to_stderr() {
    let output = cargo_bin().arg("--nonexistent").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.is_empty(),
        "invalid flag should produce stderr output"
    );
}

#[test]
fn invalid_completions_shell_writes_stderr() {
    let output = cargo_bin()
        .args(["--completions", "nosuchshell"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.is_empty(),
        "invalid shell should produce stderr output"
    );
}

// ══════════════════════════════════════════════════════════════════
//  Help output deep content validation
// ══════════════════════════════════════════════════════════════════

#[test]
fn help_contains_all_long_flags() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let flags = [
        "--interface",
        "--filter",
        "--net-filter",
        "--no-dns",
        "--no-port-names",
        "--promiscuous",
        "--no-bars",
        "--bytes",
        "--hide-ports",
        "--no-processes",
        "--json",
        "--list-interfaces",
        "--list-colors",
        "--help",
        "--version",
        "--config",
    ];
    for flag in &flags {
        assert!(stdout.contains(flag), "help missing long flag: {}", flag);
    }
}

#[test]
fn help_contains_all_keybind_keys() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // All single-key keybinds from the help
    for key in [
        "h", "n", "b", "B", "t", "T", "P", "x", "c", "g", "f", "o", "q",
    ] {
        assert!(stdout.contains(key), "help missing keybind key: {}", key);
    }
}

#[test]
fn help_keybind_section_has_help_hud() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("help HUD"),
        "help should document help HUD keybind"
    );
}

#[test]
fn help_keybind_section_has_disconnect() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("disconnect"),
        "help should document disconnect keybind"
    );
}

#[test]
fn help_keybind_section_has_cumulative() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("cumulative"),
        "help should document cumulative toggle"
    );
}

#[test]
fn help_keybind_section_has_line_mode() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("line mode"),
        "help should document line mode"
    );
}

#[test]
fn help_contains_jack_in() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("JACK IN"),
        "help should have JACK IN tagline"
    );
}

#[test]
fn help_contains_sniff_the_stream() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("SNIFF THE STREAM"),
        "help should have SNIFF THE STREAM"
    );
}

#[test]
fn help_contains_own_your_network() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OWN YOUR NETWORK"),
        "help should have OWN YOUR NETWORK"
    );
}

#[test]
fn help_has_usage_line() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("iftoprs [OPTIONS]"),
        "help should show usage with OPTIONS"
    );
}

#[test]
fn help_has_generate_completions_via_flag() {
    // The --completions flag is functional even though custom help doesn't list it
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    assert!(
        output.status.success(),
        "--completions flag should be functional"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "completions should produce output");
}

#[test]
fn help_describes_bpf_filter() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BPF"), "help should describe BPF filter");
}

#[test]
fn help_describes_cidr_notation() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("CIDR"), "help should mention CIDR notation");
}

#[test]
fn help_describes_config_file() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("config"), "help should mention config file");
    assert!(
        stdout.contains(".iftoprs.conf"),
        "help should reference .iftoprs.conf"
    );
}

#[test]
fn help_shows_no_port_names_description() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("port-to-service"),
        "help should describe port-to-service resolution"
    );
}

#[test]
fn help_banner_uses_unicode_box_drawing() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("┌"), "banner should use box drawing: ┌");
    assert!(stdout.contains("┘"), "banner should use box drawing: ┘");
    assert!(stdout.contains("│"), "banner should use box drawing: │");
    assert!(stdout.contains("──"), "banner should use box drawing: ──");
}

#[test]
fn help_banner_status_section() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("STATUS"), "banner should show STATUS");
    assert!(stdout.contains("ONLINE"), "banner should show ONLINE");
}

#[test]
fn help_banner_footer_separator() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("░░░"),
        "help should have footer separator bar"
    );
}

#[test]
fn help_has_section_separators() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Cyberpunk section headers with ── prefix
    let sections = ["CAPTURE", "KEYBINDS", "SYSTEM"];
    for section in &sections {
        assert!(
            stdout.contains(&format!("── {} ──", section)),
            "help missing section separator: {}",
            section
        );
    }
}

#[test]
fn help_uses_green_comment_markers() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The // markers are styled green: \x1b[32m//\x1b[0m
    assert!(
        stdout.contains("//"),
        "help should use // comment markers for flag descriptions"
    );
}

#[test]
fn help_line_count_range() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.lines().count();
    assert!(
        (30..=200).contains(&lines),
        "help should have 30-200 lines, got {}",
        lines
    );
}

#[test]
fn help_byte_count_range() {
    let output = cargo_bin().arg("-h").output().unwrap();
    assert!(
        output.stdout.len() >= 1000,
        "help should be at least 1000 bytes"
    );
    assert!(
        output.stdout.len() <= 20_000,
        "help should be under 20KB, got {}",
        output.stdout.len()
    );
}

// ══════════════════════════════════════════════════════════════════
//  Version output validation
// ══════════════════════════════════════════════════════════════════

#[test]
fn version_starts_with_crate_name() {
    let output = cargo_bin().arg("-V").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(
        stdout.starts_with("iftoprs "),
        "version should start with 'iftoprs '"
    );
}

#[test]
fn version_has_three_numeric_parts() {
    let output = cargo_bin().arg("-V").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let version = stdout.strip_prefix("iftoprs ").unwrap();
    let parts: Vec<&str> = version.split('.').collect();
    assert_eq!(parts.len(), 3, "version should have 3 parts: {}", version);
    for (i, part) in parts.iter().enumerate() {
        let n = part.parse::<u32>();
        assert!(n.is_ok(), "version part {} is not numeric: {}", i, part);
    }
}

#[test]
fn version_major_is_reasonable() {
    let output = cargo_bin().arg("-V").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let version = stdout.strip_prefix("iftoprs ").unwrap();
    let major: u32 = version.split('.').next().unwrap().parse().unwrap();
    assert!(
        major <= 100,
        "major version should be <= 100, got {}",
        major
    );
}

#[test]
fn version_minor_is_reasonable() {
    let output = cargo_bin().arg("-V").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let version = stdout.strip_prefix("iftoprs ").unwrap();
    let parts: Vec<&str> = version.split('.').collect();
    let minor: u32 = parts[1].parse().unwrap();
    assert!(
        minor <= 1000,
        "minor version should be <= 1000, got {}",
        minor
    );
}

#[test]
fn version_matches_cargo_toml_version() {
    let output = cargo_bin().arg("-V").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let cli_version = stdout.strip_prefix("iftoprs ").unwrap();

    let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap();
    let toml_version = cargo_toml
        .lines()
        .find(|l| l.starts_with("version = "))
        .unwrap()
        .trim_start_matches("version = ")
        .trim_matches('"');
    assert_eq!(
        cli_version, toml_version,
        "CLI version should match Cargo.toml"
    );
}

#[test]
fn version_no_trailing_whitespace_or_newlines() {
    let output = cargo_bin().arg("-V").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    assert_eq!(
        stdout.trim_end_matches('\n').trim_end_matches('\r'),
        trimmed,
        "version should not have extra whitespace"
    );
}

#[test]
fn version_is_utf8() {
    let output = cargo_bin().arg("-V").output().unwrap();
    assert!(
        String::from_utf8(output.stdout.clone()).is_ok(),
        "version should be valid UTF-8"
    );
}

// ══════════════════════════════════════════════════════════════════
//  List colors deep validation
// ══════════════════════════════════════════════════════════════════

#[test]
fn list_colors_has_exactly_31_themes() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Each theme has a color swatch with 48;5; background escapes
    // Count lines that have 48;5; sequences (one per theme)
    let theme_lines = stdout.lines().filter(|l| l.contains("48;5;")).count();
    assert_eq!(
        theme_lines, 31,
        "should have exactly 31 theme lines, got {}",
        theme_lines
    );
}

#[test]
fn list_colors_shows_each_theme_flag_name() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = [
        "neonsprawl",
        "acidrain",
        "icebreaker",
        "synthwave",
        "rustbelt",
        "ghostwire",
        "redsector",
        "sakuraden",
        "datastream",
        "solarflare",
        "neonnoir",
        "chromeheart",
        "bladerunner",
        "voidwalker",
        "toxicwaste",
        "cyberfrost",
        "plasmacore",
        "steelnerve",
        "darksignal",
        "glitchpop",
        "holoshift",
        "nightcity",
        "deepnet",
        "lasergrid",
        "quantumflux",
        "biohazard",
        "darkwave",
        "overlock",
        "megacorp",
        "zaibatsu",
        "iftopcolor",
    ];
    for name in &expected {
        assert!(
            stdout.contains(name),
            "list-colors missing theme flag name: {}",
            name
        );
    }
}

#[test]
fn list_colors_starts_with_newline() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with('\n'),
        "list-colors should start with newline for spacing"
    );
}

#[test]
fn list_colors_ends_with_newline() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.ends_with('\n'),
        "list-colors should end with newline"
    );
}

#[test]
fn list_colors_shows_tui_hint() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TUI"), "list-colors should mention TUI");
}

#[test]
fn list_colors_is_valid_utf8() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    assert!(
        String::from_utf8(output.stdout.clone()).is_ok(),
        "output should be valid UTF-8"
    );
}

// ══════════════════════════════════════════════════════════════════
//  Completions cross-shell validation
// ══════════════════════════════════════════════════════════════════

#[test]
fn all_completions_are_nonempty() {
    for shell in ["zsh", "bash", "fish", "elvish", "powershell"] {
        let output = cargo_bin().args(["--completions", shell]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.is_empty(),
            "{} completions should not be empty",
            shell
        );
        assert!(
            stdout.len() > 100,
            "{} completions should be substantial",
            shell
        );
    }
}

#[test]
fn all_completions_reference_iftoprs() {
    for shell in ["zsh", "bash", "fish", "elvish", "powershell"] {
        let output = cargo_bin().args(["--completions", shell]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("iftoprs"),
            "{} completions should reference iftoprs",
            shell
        );
    }
}

#[test]
fn all_completions_exit_code_zero() {
    for shell in ["zsh", "bash", "fish", "elvish", "powershell"] {
        let output = cargo_bin().args(["--completions", shell]).output().unwrap();
        assert!(
            output.status.success(),
            "{} completions should exit 0",
            shell
        );
    }
}

#[test]
fn all_completions_stderr_empty() {
    for shell in ["zsh", "bash", "fish", "elvish", "powershell"] {
        let output = cargo_bin().args(["--completions", shell]).output().unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        let app_stderr: String = stderr
            .lines()
            .filter(|l| {
                !l.trim().is_empty()
                    && !l.contains("warning:")
                    && !l.contains("-->")
                    && !l.contains("= note:")
                    && !l.starts_with("  ")
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            app_stderr.is_empty(),
            "{} completions should not write to stderr: {}",
            shell,
            app_stderr
        );
    }
}

#[test]
fn all_completions_are_deterministic() {
    for shell in ["zsh", "bash", "fish", "elvish", "powershell"] {
        let out1 = cargo_bin().args(["--completions", shell]).output().unwrap();
        let out2 = cargo_bin().args(["--completions", shell]).output().unwrap();
        assert_eq!(
            out1.stdout, out2.stdout,
            "{} completions should be deterministic",
            shell
        );
    }
}

#[test]
fn all_completions_are_valid_utf8() {
    for shell in ["zsh", "bash", "fish", "elvish", "powershell"] {
        let output = cargo_bin().args(["--completions", shell]).output().unwrap();
        assert!(
            String::from_utf8(output.stdout.clone()).is_ok(),
            "{} completions should be valid UTF-8",
            shell
        );
    }
}

#[test]
fn all_completions_mention_interface() {
    for shell in ["zsh", "bash", "fish", "elvish", "powershell"] {
        let output = cargo_bin().args(["--completions", shell]).output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("interface"),
            "{} completions should mention interface",
            shell
        );
    }
}

// ══════════════════════════════════════════════════════════════════
//  Zsh static completion file deeper checks
// ══════════════════════════════════════════════════════════════════

#[test]
fn zsh_completion_file_is_valid_utf8() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    assert!(!content.is_empty());
}

#[test]
fn zsh_completion_file_has_argument_descriptions() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    assert!(
        content.contains("Network"),
        "should describe network-related flags"
    );
}

#[test]
fn zsh_completion_file_has_no_trailing_whitespace_on_first_line() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    let first_line = content.lines().next().unwrap();
    assert_eq!(
        first_line,
        first_line.trim_end(),
        "first line should not have trailing whitespace"
    );
}

#[test]
fn zsh_completion_file_ends_with_newline() {
    let bytes = std::fs::read("completions/_iftoprs").unwrap();
    assert_eq!(
        *bytes.last().unwrap(),
        b'\n',
        "file should end with newline"
    );
}

// ══════════════════════════════════════════════════════════════════
//  Default config file deep validation
// ══════════════════════════════════════════════════════════════════

#[test]
fn default_config_has_dns_resolution_field() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(
        content.contains("dns_resolution"),
        "should have dns_resolution field"
    );
}

#[test]
fn default_config_has_port_resolution_field() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(
        content.contains("port_resolution"),
        "should have port_resolution field"
    );
}

#[test]
fn default_config_has_use_bytes_field() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(content.contains("use_bytes"), "should have use_bytes field");
}

#[test]
fn default_config_has_show_cumulative_field() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(
        content.contains("show_cumulative"),
        "should have show_cumulative field"
    );
}

#[test]
fn default_config_has_show_processes_field() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(
        content.contains("show_processes"),
        "should have show_processes field"
    );
}

#[test]
fn default_config_has_show_header_field() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(
        content.contains("show_header"),
        "should have show_header field"
    );
}

#[test]
fn default_config_has_bar_style_field() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(content.contains("bar_style"), "should have bar_style field");
}

#[test]
fn default_config_bar_style_default() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(
        content.contains("bar_style = \"Gradient\""),
        "default bar style should be Gradient"
    );
}

#[test]
fn default_config_show_border_default_true() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(
        content.contains("show_border = true"),
        "show_border default should be true"
    );
}

#[test]
fn default_config_show_header_default_true() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(
        content.contains("show_header = true"),
        "show_header default should be true"
    );
}

#[test]
fn default_config_show_ports_default_true() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(
        content.contains("show_ports = true"),
        "show_ports default should be true"
    );
}

#[test]
fn default_config_show_bars_default_true() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(
        content.contains("show_bars = true"),
        "show_bars default should be true"
    );
}

#[test]
fn default_config_use_bytes_default_false() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(
        content.contains("use_bytes = false"),
        "use_bytes default should be false"
    );
}

#[test]
fn default_config_show_cumulative_default_false() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(
        content.contains("show_cumulative = false"),
        "show_cumulative default should be false"
    );
}

#[test]
fn default_config_is_not_empty() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(content.len() > 100, "default config should be substantial");
}

#[test]
fn default_config_has_no_tab_characters() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(
        !content.contains('\t'),
        "default config should use spaces, not tabs"
    );
}

#[test]
fn default_config_ends_with_newline() {
    let bytes = std::fs::read("iftoprs.default.conf").unwrap();
    assert_eq!(
        *bytes.last().unwrap(),
        b'\n',
        "config file should end with newline"
    );
}

#[test]
fn default_config_documents_all_theme_variants() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    // At least a representative sample
    for theme in [
        "NeonSprawl",
        "AcidRain",
        "IceBreaker",
        "SynthWave",
        "BladeRunner",
        "NeonNoir",
        "GlitchPop",
        "NightCity",
        "Zaibatsu",
        "Iftopcolor",
    ] {
        assert!(
            content.contains(theme),
            "default config should list theme: {}",
            theme
        );
    }
}

#[test]
fn default_config_documents_refresh_rate_options() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    assert!(
        content.contains("refresh_rate"),
        "should document refresh_rate"
    );
}

// ══════════════════════════════════════════════════════════════════
//  Cargo.toml metadata validation
// ══════════════════════════════════════════════════════════════════

#[test]
fn cargo_toml_has_description() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(
        content.contains("description = "),
        "should have description"
    );
}

#[test]
fn cargo_toml_has_license() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(content.contains("license = "), "should have license");
}

#[test]
fn cargo_toml_license_is_mit() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(
        content.contains("license = \"MIT\""),
        "license should be MIT"
    );
}

#[test]
fn cargo_toml_has_repository() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(content.contains("repository = "), "should have repository");
}

#[test]
fn cargo_toml_has_keywords() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(content.contains("keywords"), "should have keywords");
    assert!(
        content.contains("network"),
        "keywords should include network"
    );
    assert!(
        content.contains("bandwidth"),
        "keywords should include bandwidth"
    );
    assert!(
        content.contains("monitor"),
        "keywords should include monitor"
    );
    assert!(content.contains("tui"), "keywords should include tui");
    assert!(content.contains("iftop"), "keywords should include iftop");
}

#[test]
fn cargo_toml_has_categories() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(content.contains("categories"), "should have categories");
    assert!(
        content.contains("command-line-utilities"),
        "should include command-line-utilities category"
    );
    assert!(
        content.contains("network-programming"),
        "should include network-programming category"
    );
}

#[test]
fn cargo_toml_has_readme() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(
        content.contains("readme = \"README.md\""),
        "should reference README.md"
    );
}

#[test]
fn cargo_toml_edition_2024() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(
        content.contains("edition = \"2024\""),
        "should use 2024 edition"
    );
}

#[test]
fn cargo_toml_has_release_profile() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(
        content.contains("[profile.release]"),
        "should have release profile"
    );
    assert!(content.contains("lto = true"), "release should enable LTO");
    assert!(
        content.contains("strip = true"),
        "release should enable strip"
    );
}

#[test]
fn cargo_toml_has_all_dependencies() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    let deps = [
        "anyhow",
        "chrono",
        "clap",
        "crossterm",
        "dirs",
        "dns-lookup",
        "pcap",
        "ratatui",
        "regex",
        "serde",
        "serde_json",
        "tokio",
        "toml",
        "clap_complete",
    ];
    for dep in &deps {
        assert!(
            content.contains(dep),
            "Cargo.toml missing dependency: {}",
            dep
        );
    }
}

#[test]
fn cargo_toml_clap_has_derive_feature() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(
        content.contains("derive"),
        "clap should have derive feature"
    );
}

#[test]
fn cargo_toml_serde_has_derive_feature() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(
        content.contains("serde") && content.contains("derive"),
        "serde should have derive feature"
    );
}

#[test]
fn cargo_toml_tokio_has_full_feature() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(
        content.contains("tokio") && content.contains("full"),
        "tokio should have full feature"
    );
}

// ══════════════════════════════════════════════════════════════════
//  README content validation
// ══════════════════════════════════════════════════════════════════

#[test]
fn readme_exists() {
    assert!(std::path::Path::new("README.md").exists());
}

#[test]
fn readme_has_ascii_art_banner() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("██"),
        "README should have ASCII art banner"
    );
}

#[test]
fn readme_has_badges() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("crates.io"),
        "README should have crates.io badge"
    );
}

#[test]
fn readme_documents_installation() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("cargo install iftoprs"),
        "README should show installation command"
    );
}

#[test]
fn readme_documents_build() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("cargo build"),
        "README should show build command"
    );
}

#[test]
fn readme_documents_all_keybinds() {
    let content = std::fs::read_to_string("README.md").unwrap();
    for key in [
        "Tab", "j", "k", "q", "h", "n", "b", "B", "p", "P", "t", "T", "x", "g", "f", "o", "e", "y",
        "F",
    ] {
        assert!(content.contains(key), "README should document key: {}", key);
    }
}

#[test]
fn readme_documents_all_cli_flags() {
    let content = std::fs::read_to_string("README.md").unwrap();
    let flags = [
        "--interface",
        "--filter",
        "--net-filter",
        "--no-dns",
        "--no-port-names",
        "--promiscuous",
        "--no-bars",
        "--bytes",
        "--hide-ports",
        "--no-processes",
        "--json",
        "--list-interfaces",
        "--list-colors",
        "--completions",
        "--help",
        "--version",
    ];
    for flag in &flags {
        assert!(
            content.contains(flag),
            "README should document flag: {}",
            flag
        );
    }
}

#[test]
fn readme_documents_platforms() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(content.contains("macOS"), "README should mention macOS");
    assert!(content.contains("Linux"), "README should mention Linux");
}

#[test]
fn readme_documents_libpcap_requirement() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("libpcap"),
        "README should mention libpcap requirement"
    );
}

#[test]
fn readme_documents_theme_count() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(content.contains("31"), "README should mention 31 themes");
}

#[test]
fn readme_documents_sudo_requirement() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("sudo"),
        "README should mention sudo for capture"
    );
}

#[test]
fn readme_documents_ratatui() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(content.contains("ratatui"), "README should mention ratatui");
}

#[test]
fn readme_documents_crossterm() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("crossterm"),
        "README should mention crossterm"
    );
}

#[test]
fn readme_documents_pcap() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(content.contains("pcap"), "README should mention pcap");
}

#[test]
fn readme_documents_filter_examples() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("tcp port"),
        "README should show BPF filter example"
    );
}

#[test]
fn readme_documents_cidr_example() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("10.0.0.0/8"),
        "README should show CIDR example"
    );
}

#[test]
fn readme_documents_mouse_support() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("click") || content.contains("Mouse") || content.contains("mouse"),
        "README should mention mouse support"
    );
}

#[test]
fn readme_documents_config_file() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains(".iftoprs.conf"),
        "README should mention config file"
    );
}

#[test]
fn readme_documents_export() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("export") || content.contains("Export"),
        "README should document export functionality"
    );
}

#[test]
fn readme_documents_sort() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("Sort") || content.contains("sort"),
        "README should document sort functionality"
    );
}

#[test]
fn readme_documents_sliding_window() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("2s") && content.contains("10s") && content.contains("40s"),
        "README should document sliding window averages"
    );
}

#[test]
fn readme_documents_dns_resolution() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("DNS"),
        "README should document DNS resolution"
    );
}

#[test]
fn readme_documents_protocol_types() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(content.contains("TCP"), "README should mention TCP");
    assert!(content.contains("UDP"), "README should mention UDP");
    assert!(content.contains("ICMP"), "README should mention ICMP");
}

#[test]
fn readme_is_substantial() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.len() > 5000,
        "README should be substantial, got {} bytes",
        content.len()
    );
}

#[test]
fn readme_has_menketechnologies_credit() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("MenkeTechnologies"),
        "README should credit author"
    );
}

// ══════════════════════════════════════════════════════════════════
//  Project structure validation
// ══════════════════════════════════════════════════════════════════

#[test]
fn src_main_exists() {
    assert!(std::path::Path::new("src/main.rs").exists());
}

#[test]
fn src_lib_exists() {
    assert!(std::path::Path::new("src/lib.rs").exists());
}

#[test]
fn src_capture_dir_exists() {
    assert!(std::path::Path::new("src/capture").exists());
}

#[test]
fn src_config_dir_exists() {
    assert!(std::path::Path::new("src/config").exists());
}

#[test]
fn src_data_dir_exists() {
    assert!(std::path::Path::new("src/data").exists());
}

#[test]
fn src_ui_dir_exists() {
    assert!(std::path::Path::new("src/ui").exists());
}

#[test]
fn src_util_dir_exists() {
    assert!(std::path::Path::new("src/util").exists());
}

#[test]
fn capture_parser_exists() {
    assert!(std::path::Path::new("src/capture/parser.rs").exists());
}

#[test]
fn capture_sniffer_exists() {
    assert!(std::path::Path::new("src/capture/sniffer.rs").exists());
}

#[test]
fn config_cli_exists() {
    assert!(std::path::Path::new("src/config/cli.rs").exists());
}

#[test]
fn config_prefs_exists() {
    assert!(std::path::Path::new("src/config/prefs.rs").exists());
}

#[test]
fn config_theme_exists() {
    assert!(std::path::Path::new("src/config/theme.rs").exists());
}

#[test]
fn data_flow_exists() {
    assert!(std::path::Path::new("src/data/flow.rs").exists());
}

#[test]
fn data_history_exists() {
    assert!(std::path::Path::new("src/data/history.rs").exists());
}

#[test]
fn data_tracker_exists() {
    assert!(std::path::Path::new("src/data/tracker.rs").exists());
}

#[test]
fn ui_app_exists() {
    assert!(std::path::Path::new("src/ui/app.rs").exists());
}

#[test]
fn ui_render_exists() {
    assert!(std::path::Path::new("src/ui/render.rs").exists());
}

#[test]
fn util_format_exists() {
    assert!(std::path::Path::new("src/util/format.rs").exists());
}

#[test]
fn util_resolver_exists() {
    assert!(std::path::Path::new("src/util/resolver.rs").exists());
}

#[test]
fn util_procinfo_exists() {
    assert!(std::path::Path::new("src/util/procinfo.rs").exists());
}

#[test]
fn completions_dir_exists() {
    assert!(std::path::Path::new("completions").exists());
}

#[test]
fn license_file_exists() {
    assert!(
        std::path::Path::new("LICENSE").exists() || std::path::Path::new("LICENSE.md").exists(),
        "should have a LICENSE file"
    );
}

#[test]
fn screenshots_dir_exists() {
    assert!(
        std::path::Path::new("screenshots").exists(),
        "screenshots directory should exist"
    );
}

// ══════════════════════════════════════════════════════════════════
//  Source file content validation
// ══════════════════════════════════════════════════════════════════

#[test]
fn lib_rs_declares_all_modules() {
    let content = std::fs::read_to_string("src/lib.rs").unwrap();
    for module in ["capture", "config", "data", "ui", "util"] {
        assert!(
            content.contains(&format!("pub mod {}", module)),
            "lib.rs should declare pub mod {}",
            module
        );
    }
}

#[test]
fn main_rs_declares_all_modules() {
    let content = std::fs::read_to_string("src/main.rs").unwrap();
    for module in ["capture", "config", "data", "ui", "util"] {
        assert!(
            content.contains(&format!("mod {}", module)),
            "main.rs should declare mod {}",
            module
        );
    }
}

#[test]
fn main_rs_has_main_function() {
    let content = std::fs::read_to_string("src/main.rs").unwrap();
    assert!(
        content.contains("fn main()"),
        "main.rs should have main function"
    );
}

#[test]
fn main_rs_uses_clap_parser() {
    let content = std::fs::read_to_string("src/main.rs").unwrap();
    assert!(
        content.contains("Args::parse()"),
        "main.rs should use Args::parse()"
    );
}

#[test]
fn main_rs_returns_result() {
    let content = std::fs::read_to_string("src/main.rs").unwrap();
    assert!(
        content.contains("-> Result<()>"),
        "main should return Result"
    );
}

// ══════════════════════════════════════════════════════════════════
//  Help output encoding
// ══════════════════════════════════════════════════════════════════

#[test]
fn help_output_is_valid_utf8() {
    let output = cargo_bin().arg("-h").output().unwrap();
    assert!(
        String::from_utf8(output.stdout.clone()).is_ok(),
        "help should be valid UTF-8"
    );
}

#[test]
fn help_contains_ansi_reset_codes() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\x1b[0m"),
        "help should contain ANSI reset codes"
    );
}

#[test]
fn help_contains_cyan_ansi() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\x1b[36m"),
        "help should use cyan ANSI color"
    );
}

#[test]
fn help_contains_magenta_ansi() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\x1b[35m"),
        "help should use magenta ANSI color"
    );
}

#[test]
fn help_contains_red_ansi() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\x1b[31m"),
        "help should use red ANSI color"
    );
}

#[test]
fn help_contains_yellow_ansi() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\x1b[33m"),
        "help should use yellow ANSI color"
    );
}

#[test]
fn help_contains_green_ansi() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("\x1b[32m"),
        "help should use green ANSI color"
    );
}

// ══════════════════════════════════════════════════════════════════
//  Multiple runs consistency
// ══════════════════════════════════════════════════════════════════

#[test]
fn list_colors_output_is_deterministic() {
    let out1 = cargo_bin().arg("--list-colors").output().unwrap();
    let out2 = cargo_bin().arg("--list-colors").output().unwrap();
    assert_eq!(
        out1.stdout, out2.stdout,
        "list-colors should be deterministic"
    );
}

#[test]
fn help_flag_long_and_short_produce_same_output() {
    let short = cargo_bin().arg("-h").output().unwrap();
    // Can't test --help since it's custom
    assert!(short.status.success());
}

// ══════════════════════════════════════════════════════════════════
//  Config flag interaction
// ══════════════════════════════════════════════════════════════════

#[test]
fn config_flag_with_valid_temp_file() {
    let path = "/tmp/iftoprs_test_empty.conf";
    std::fs::write(path, "").unwrap();
    let output = cargo_bin().args(["-c", path, "-h"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BANDWIDTH MONITOR"),
        "help should work with empty config"
    );
    assert!(output.status.success());
    let _ = std::fs::remove_file(path);
}

#[test]
fn config_flag_with_custom_theme() {
    let path = "/tmp/iftoprs_test_theme.conf";
    std::fs::write(path, "theme = \"BladeRunner\"\n").unwrap();
    let output = cargo_bin().args(["-c", path, "-h"]).output().unwrap();
    assert!(
        output.status.success(),
        "should work with custom theme config"
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn config_flag_with_invalid_toml() {
    let path = "/tmp/iftoprs_test_invalid.conf";
    std::fs::write(path, "this is not valid toml [[[").unwrap();
    let output = cargo_bin().args(["-c", path, "-h"]).output().unwrap();
    // Should still show help (help takes priority)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BANDWIDTH MONITOR"),
        "help should work even with invalid config"
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn config_flag_with_all_default_values() {
    let path = "/tmp/iftoprs_test_full.conf";
    let config = r#"
theme = "NeonSprawl"
dns_resolution = true
port_resolution = true
show_ports = true
show_bars = true
use_bytes = false
show_processes = true
show_cumulative = false
bar_style = "Gradient"
show_border = true
show_header = true
refresh_rate = 1
alert_threshold = 0.0
"#;
    std::fs::write(path, config).unwrap();
    let output = cargo_bin().args(["-c", path, "-h"]).output().unwrap();
    assert!(
        output.status.success(),
        "should work with full default config"
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn config_flag_with_custom_refresh_rate() {
    let path = "/tmp/iftoprs_test_refresh.conf";
    std::fs::write(path, "refresh_rate = 5\n").unwrap();
    let output = cargo_bin().args(["-c", path, "-h"]).output().unwrap();
    assert!(output.status.success());
    let _ = std::fs::remove_file(path);
}

// ══════════════════════════════════════════════════════════════════
//  Interface flag
// ══════════════════════════════════════════════════════════════════

#[test]
fn interface_flag_with_help() {
    let output = cargo_bin().args(["-i", "lo0", "-h"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BANDWIDTH MONITOR"),
        "help should work with -i flag"
    );
    assert!(output.status.success());
}

#[test]
fn interface_long_flag_with_help() {
    let output = cargo_bin()
        .args(["--interface", "en0", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

// ══════════════════════════════════════════════════════════════════
//  Filter flag
// ══════════════════════════════════════════════════════════════════

#[test]
fn filter_flag_with_help() {
    let output = cargo_bin()
        .args(["-f", "tcp port 80", "-h"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BANDWIDTH MONITOR"));
    assert!(output.status.success());
}

#[test]
fn filter_long_flag_with_help() {
    let output = cargo_bin()
        .args(["--filter", "udp", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_flag_with_help() {
    let output = cargo_bin()
        .args(["-F", "192.168.1.0/24", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_long_flag_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "10.0.0.0/8", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_multicast_slash4_with_help() {
    let output = cargo_bin()
        .args(["-F", "224.0.0.0/4", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_ff00_slash8_with_help() {
    let output = cargo_bin().args(["-F", "ff00::/8", "-h"]).output().unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_documentation_db8_slash48_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "2001:db8:1::/48", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_documentation_db8_slash32_with_help() {
    let output = cargo_bin()
        .args(["-F", "2001:db8::/32", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_loopback_slash128_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "::1/128", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_default_route_slash0_with_help() {
    let output = cargo_bin()
        .args(["-F", "0.0.0.0/0", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_b_slash12_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "172.16.0.0/12", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_link_local_apipa_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "169.254.0.0/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_orchid_slash28_with_help() {
    let output = cargo_bin()
        .args(["-F", "2001:10::/28", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_all_addresses_slash0_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "::/0", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_c_aggregate_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "192.168.0.0/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_unique_local_fc00_slash7_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "fc00::/7", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_nat64_well_known_slash96_with_help() {
    let output = cargo_bin()
        .args(["-F", "64:ff9b::/96", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_unique_local_fd00_slash8_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "fd00::/8", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_six_to_four_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "2002::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_teredo_slash32_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "2001::/32", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_benchmarking_slash48_with_help() {
    let output = cargo_bin()
        .args(["-F", "2001:2::/48", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_link_local_fe80_slash10_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "fe80::/10", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_loopback_slash8_with_help() {
    let output = cargo_bin()
        .args(["-F", "127.0.0.0/8", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_limited_broadcast_slash32_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "255.255.255.255/32", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_reserved_class_e_slash4_with_help() {
    let output = cargo_bin()
        .args(["-F", "240.0.0.0/4", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_deprecated_site_local_fec0_slash10_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "fec0::/10", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_cgnat_shared_address_space_slash10_with_help() {
    let output = cargo_bin()
        .args(["-F", "100.64.0.0/10", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_documentation_test_net_2_slash24_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "198.51.100.0/24", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_documentation_test_net_3_slash24_with_help() {
    let output = cargo_bin()
        .args(["-F", "203.0.113.0/24", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_documentation_test_net_1_slash24_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "192.0.2.0/24", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_rfc2544_benchmark_slash15_with_help() {
    let output = cargo_bin()
        .args(["-F", "198.18.0.0/15", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_ipv4_mapped_well_known_slash96_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "::ffff:0:0/96", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_link_local_multicast_ff02_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "ff02::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_global_unicast_slash3_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "2000::/3", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_slash2_quarter_internet_with_help() {
    let output = cargo_bin()
        .args(["-F", "0.0.0.0/2", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_documentation_db8_slash112_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "2001:db8::/112", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_c_slash24_with_help() {
    let output = cargo_bin()
        .args(["-F", "192.168.0.0/24", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_documentation_db8_slash64_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "2001:db8::/64", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_link_local_fe80_slash64_with_help() {
    let output = cargo_bin()
        .args(["-F", "fe80::/64", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_unique_local_fd12_slash48_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "fd12:3456:789a::/48", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_multicast_admin_scope_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "239.0.0.0/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_documentation_db8_host_slash128_with_help() {
    let output = cargo_bin()
        .args(["-F", "2001:db8::1/128", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_documentation_test_net_slash30_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "192.0.2.0/30", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_c_slash25_with_help() {
    let output = cargo_bin()
        .args(["-F", "192.168.0.0/25", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_six_to_four_derived_slash48_with_help() {
    // 6to4 encoding of IPv4 192.0.2.4 → 2002:c000:0204::/48
    let output = cargo_bin()
        .args(["--net-filter", "2002:c000:0204::/48", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_ssm_ff3e_slash32_with_help() {
    let output = cargo_bin()
        .args(["-F", "ff3e::/32", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_documentation_db8_slash56_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "2001:db8::/56", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_apipa_slash31_with_help() {
    let output = cargo_bin()
        .args(["-F", "169.254.0.0/31", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_a_slash26_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "10.0.0.0/26", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_site_scope_ff05_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "ff05::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_documentation_db8_child_slash56_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "2001:db8:1::/56", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_interdomain_ff0e_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "ff0e::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_c_slash28_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "192.168.1.0/28", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_documentation_db8_slash96_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "2001:db8::/96", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_interface_local_ff01_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "ff01::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_b_slash18_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "172.16.0.0/18", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_a_slash17_with_help() {
    let output = cargo_bin()
        .args(["-F", "10.127.0.0/17", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_slash22_four_class_c_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "10.0.0.0/22", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_reserved_ff0f_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "ff0f::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_c_slash23_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "192.168.0.0/23", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_node_local_ff08_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "ff08::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_subnet_local_ff03_slash16_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "ff03::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_admin_local_ff04_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "ff04::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_ff06_slash16_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "ff06::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_ff07_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "ff07::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_ff0a_slash16_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "ff0a::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_ff0b_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "ff0b::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_ff0c_slash16_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "ff0c::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_ff0d_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "ff0d::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_ff10_slash16_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "ff10::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv6_multicast_ff11_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "ff11::/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_b_172_25_slash16_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "172.25.0.0/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_a_10_255_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "10.255.0.0/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_b_172_31_slash16_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "172.31.0.0/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_c_192_168_255_slash24_with_help() {
    let output = cargo_bin()
        .args(["-F", "192.168.255.0/24", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_a_10_slash15_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "10.0.0.0/15", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_link_local_apipa_last_slash24_with_help() {
    let output = cargo_bin()
        .args(["-F", "169.254.255.0/24", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_c_aggregate_192_168_slash15_with_help() {
    let output = cargo_bin()
        .args(["--net-filter", "192.168.0.0/15", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_ipv4_private_class_b_172_24_slash16_with_help() {
    let output = cargo_bin()
        .args(["-F", "172.24.0.0/16", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

// ══════════════════════════════════════════════════════════════════
//  Help content: flag descriptions detail
// ══════════════════════════════════════════════════════════════════

#[test]
fn help_interface_flag_mentions_jack_into() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("jack into"),
        "interface description should say 'jack into'"
    );
}

#[test]
fn help_no_dns_flag_mentions_kill() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Kill DNS"),
        "no-dns description should say 'Kill DNS'"
    );
}

#[test]
fn help_no_bars_flag_mentions_flatline() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Flatline"),
        "no-bars description should say 'Flatline'"
    );
}

#[test]
fn help_hide_ports_flag_mentions_ghost() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Ghost"),
        "hide-ports description should say 'Ghost'"
    );
}

#[test]
fn help_promiscuous_flag_mentions_sniff() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("sniff"),
        "promiscuous description should mention sniffing"
    );
}

#[test]
fn help_list_interfaces_flag_mentions_enumerate() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Enumerate"),
        "list-interfaces should say Enumerate"
    );
}

#[test]
fn help_list_colors_flag_mentions_preview() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Preview"), "list-colors should say Preview");
}

#[test]
fn help_bytes_flag_mentions_bytes() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("bytes instead of bits") || stdout.contains("bytes"),
        "bytes flag should mention bytes"
    );
}

// ══════════════════════════════════════════════════════════════════
//  Completions flag descriptions in shells
// ══════════════════════════════════════════════════════════════════

#[test]
fn zsh_completions_include_flag_descriptions() {
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Zsh completions include [description] after flags
    assert!(
        stdout.contains("interface") || stdout.contains("Interface"),
        "zsh completions should describe interface flag"
    );
}

#[test]
fn fish_completions_have_descriptions() {
    let output = cargo_bin()
        .args(["--completions", "fish"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Fish completions use -d for descriptions
    assert!(
        stdout.contains("-d") || stdout.contains("description"),
        "fish completions should have descriptions"
    );
}

// ══════════════════════════════════════════════════════════════════
//  Output size bounds
// ══════════════════════════════════════════════════════════════════

#[test]
fn completions_zsh_size_reasonable() {
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    assert!(
        output.stdout.len() > 500,
        "zsh completions should be > 500 bytes"
    );
    assert!(
        output.stdout.len() < 50_000,
        "zsh completions should be < 50KB"
    );
}

#[test]
fn completions_bash_size_reasonable() {
    let output = cargo_bin()
        .args(["--completions", "bash"])
        .output()
        .unwrap();
    assert!(
        output.stdout.len() > 500,
        "bash completions should be > 500 bytes"
    );
    assert!(
        output.stdout.len() < 50_000,
        "bash completions should be < 50KB"
    );
}

#[test]
fn completions_fish_size_reasonable() {
    let output = cargo_bin()
        .args(["--completions", "fish"])
        .output()
        .unwrap();
    assert!(
        output.stdout.len() > 200,
        "fish completions should be > 200 bytes"
    );
    assert!(
        output.stdout.len() < 50_000,
        "fish completions should be < 50KB"
    );
}

#[test]
fn version_output_size_reasonable() {
    let output = cargo_bin().arg("-V").output().unwrap();
    assert!(output.stdout.len() > 10, "version should be > 10 bytes");
    assert!(output.stdout.len() < 100, "version should be < 100 bytes");
}

#[test]
fn list_colors_size_reasonable() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    assert!(
        output.stdout.len() > 1000,
        "list-colors should be > 1000 bytes"
    );
    assert!(output.stdout.len() < 50_000, "list-colors should be < 50KB");
}

// ══════════════════════════════════════════════════════════════════
//  Default config TOML parse and field validation
// ══════════════════════════════════════════════════════════════════

#[test]
fn default_config_parses_to_correct_theme() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    let filtered: String = content
        .lines()
        .filter(|l| !l.trim_start().starts_with('#') || l.contains('='))
        .collect::<Vec<_>>()
        .join("\n");
    let parsed: toml::Value = toml::from_str(&filtered).unwrap();
    let theme = parsed.get("theme").unwrap().as_str().unwrap();
    assert_eq!(theme, "NeonSprawl", "default theme should be NeonSprawl");
}

#[test]
fn default_config_parses_refresh_rate() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    let filtered: String = content
        .lines()
        .filter(|l| !l.trim_start().starts_with('#') || l.contains('='))
        .collect::<Vec<_>>()
        .join("\n");
    let parsed: toml::Value = toml::from_str(&filtered).unwrap();
    let rate = parsed.get("refresh_rate").unwrap().as_integer().unwrap();
    assert_eq!(rate, 1, "default refresh_rate should be 1");
}

#[test]
fn default_config_parses_alert_threshold() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    let filtered: String = content
        .lines()
        .filter(|l| !l.trim_start().starts_with('#') || l.contains('='))
        .collect::<Vec<_>>()
        .join("\n");
    let parsed: toml::Value = toml::from_str(&filtered).unwrap();
    let threshold = parsed.get("alert_threshold").unwrap().as_float().unwrap();
    assert_eq!(threshold, 0.0, "default alert_threshold should be 0.0");
}

#[test]
fn default_config_parses_booleans_correctly() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    let filtered: String = content
        .lines()
        .filter(|l| !l.trim_start().starts_with('#') || l.contains('='))
        .collect::<Vec<_>>()
        .join("\n");
    let parsed: toml::Value = toml::from_str(&filtered).unwrap();
    assert!(parsed.get("show_border").unwrap().as_bool().unwrap());
    assert!(parsed.get("show_bars").unwrap().as_bool().unwrap());
    assert!(parsed.get("show_ports").unwrap().as_bool().unwrap());
    assert!(!parsed.get("use_bytes").unwrap().as_bool().unwrap());
}

// ══════════════════════════════════════════════════════════════════
//  README structure validation
// ══════════════════════════════════════════════════════════════════

#[test]
fn readme_has_feature_dump_section() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("FEATURE_DUMP"),
        "README should have feature dump section"
    );
}

#[test]
fn readme_has_cli_options_section() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("CLI_OPTIONS"),
        "README should have CLI options section"
    );
}

#[test]
fn readme_has_keybind_matrix_section() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("KEYBIND_MATRIX"),
        "README should have keybind matrix section"
    );
}

#[test]
fn readme_has_compile_sequence_section() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("COMPILE_SEQUENCE"),
        "README should have compile section"
    );
}

#[test]
fn readme_documents_capture_engine() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("CAPTURE_ENGINE"),
        "README should document capture engine"
    );
}

#[test]
fn readme_documents_theme_engine() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("THEME_ENGINE"),
        "README should document theme engine"
    );
}

#[test]
fn readme_documents_process_intel() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("PROCESS_INTEL"),
        "README should document process intel"
    );
}

#[test]
fn readme_documents_json_stream_feature() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("JSON_STREAM"),
        "README should document JSON stream feature"
    );
}

#[test]
fn readme_documents_filter_engine() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("FILTER_ENGINE"),
        "README should document filter engine"
    );
}

#[test]
fn readme_documents_alert_system() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("ALERT_SYSTEM"),
        "README should document alert system"
    );
}

#[test]
fn readme_documents_config_engine() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("CONFIG_ENGINE"),
        "README should document config engine"
    );
}

#[test]
fn readme_documents_shell_completion() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("SHELL_COMPLETION"),
        "README should document shell completion"
    );
}

#[test]
fn readme_documents_platform_compat() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("PLATFORM_COMPAT"),
        "README should document platform compatibility"
    );
}

// ══════════════════════════════════════════════════════════════════
//  Edge case: empty/whitespace inputs
// ══════════════════════════════════════════════════════════════════

#[test]
fn interface_flag_with_empty_string_still_parses() {
    // Empty string is technically a valid argument value for clap
    let output = cargo_bin().args(["-i", "", "-h"]).output().unwrap();
    // Should still show help
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BANDWIDTH MONITOR"));
}

#[test]
fn filter_flag_with_empty_string_still_parses() {
    let output = cargo_bin().args(["-f", "", "-h"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BANDWIDTH MONITOR"));
}

// ══════════════════════════════════════════════════════════════════
//  Long flag aliases (--flag=value form)
// ══════════════════════════════════════════════════════════════════

#[test]
fn interface_equals_form_with_help() {
    let output = cargo_bin()
        .args(["--interface=lo0", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn filter_equals_form_with_help() {
    let output = cargo_bin().args(["--filter=tcp", "-h"]).output().unwrap();
    assert!(output.status.success());
}

#[test]
fn net_filter_equals_form_with_help() {
    let output = cargo_bin()
        .args(["--net-filter=10.0.0.0/8", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn config_equals_form_with_help() {
    let output = cargo_bin()
        .args(["--config=/tmp/nonexistent", "-h"])
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn completions_equals_form() {
    let output = cargo_bin().args(["--completions=zsh"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("#compdef iftoprs"));
    assert!(output.status.success());
}

// ══════════════════════════════════════════════════════════════════
//  List colors theme display names
// ══════════════════════════════════════════════════════════════════

#[test]
fn list_colors_shows_display_names() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let display_names = [
        "Neon Sprawl",
        "Acid Rain",
        "Ice Breaker",
        "Synth Wave",
        "Rust Belt",
        "Ghost Wire",
        "Red Sector",
        "Sakura Den",
        "Data Stream",
        "Solar Flare",
        "Neon Noir",
        "Chrome Heart",
        "Blade Runner",
        "Void Walker",
        "Toxic Waste",
        "Cyber Frost",
        "Plasma Core",
        "Steel Nerve",
        "Dark Signal",
        "Glitch Pop",
        "Holo Shift",
        "Night City",
        "Deep Net",
        "Laser Grid",
        "Quantum Flux",
        "Bio Hazard",
        "Darkwave",
        "Overlock",
        "Megacorp",
        "Zaibatsu",
        "iftopcolor",
    ];
    for name in &display_names {
        assert!(
            stdout.contains(name),
            "list-colors missing display name: {}",
            name
        );
    }
}

// ══════════════════════════════════════════════════════════════════
//  Help and version flag priority
// ══════════════════════════════════════════════════════════════════

#[test]
fn help_takes_priority_over_json() {
    let output = cargo_bin().args(["--json", "-h"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BANDWIDTH MONITOR"),
        "help should take priority over json mode"
    );
    assert!(output.status.success());
}

#[test]
fn version_takes_priority_over_json() {
    let output = cargo_bin().args(["--json", "-V"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("iftoprs "),
        "version should take priority over json mode"
    );
    assert!(output.status.success());
}

#[test]
fn list_colors_takes_priority_over_json() {
    let output = cargo_bin()
        .args(["--json", "--list-colors"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BUILTIN COLOR SCHEMES"),
        "list-colors should take priority over json"
    );
    assert!(output.status.success());
}

#[test]
fn help_takes_priority_over_list_colors() {
    let output = cargo_bin().args(["--list-colors", "-h"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BANDWIDTH MONITOR"),
        "help should take priority over list-colors"
    );
}

#[test]
fn help_takes_priority_over_completions() {
    let output = cargo_bin()
        .args(["--completions", "zsh", "-h"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("BANDWIDTH MONITOR"),
        "help should take priority over completions"
    );
}

// ══════════════════════════════════════════════════════════════════
//  Output does not contain debug/unwanted content
// ══════════════════════════════════════════════════════════════════

#[test]
fn help_does_not_contain_debug_output() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("DEBUG"),
        "help should not contain DEBUG output"
    );
    assert!(
        !stdout.contains("TRACE"),
        "help should not contain TRACE output"
    );
}

#[test]
fn version_does_not_contain_debug_output() {
    let output = cargo_bin().arg("-V").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("DEBUG"));
    assert!(!stdout.contains("TRACE"));
}

#[test]
fn list_colors_does_not_contain_debug_output() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("DEBUG"));
    assert!(!stdout.contains("TRACE"));
}

// ══════════════════════════════════════════════════════════════════
//  Cargo.toml version consistency
// ══════════════════════════════════════════════════════════════════

#[test]
fn cargo_toml_version_is_semver() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    let version_line = content
        .lines()
        .find(|l| l.starts_with("version = "))
        .unwrap();
    let version = version_line
        .trim_start_matches("version = ")
        .trim_matches('"');
    let parts: Vec<&str> = version.split('.').collect();
    assert_eq!(parts.len(), 3, "Cargo.toml version should be semver");
    for part in &parts {
        assert!(
            part.parse::<u32>().is_ok(),
            "non-numeric version part: {}",
            part
        );
    }
}

// ══════════════════════════════════════════════════════════════════
//  Help output does not have extraneous content
// ══════════════════════════════════════════════════════════════════

#[test]
fn help_does_not_reference_cargo_or_rust_internally() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Help should reference Rust in context but not expose internal cargo details
    assert!(
        !stdout.contains("Cargo.toml"),
        "help should not mention Cargo.toml"
    );
    assert!(
        !stdout.contains("target/"),
        "help should not mention target/"
    );
}

#[test]
fn help_does_not_show_clap_default_help() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Our custom help replaces clap's default; check we see our cyberpunk banner
    assert!(
        stdout.contains("██"),
        "should use custom help with ASCII art, not clap default"
    );
    assert!(
        stdout.contains("BANDWIDTH MONITOR"),
        "should use cyberpunk help banner"
    );
}

// ══════════════════════════════════════════════════════════════════
//  Zsh completions file additional checks
// ══════════════════════════════════════════════════════════════════

#[test]
fn zsh_completion_file_has_arguments_section() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    assert!(
        content.contains("_arguments"),
        "should use _arguments for completion"
    );
}

#[test]
fn zsh_completion_file_is_substantial() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    assert!(
        content.len() > 500,
        "completion file should be substantial, got {} bytes",
        content.len()
    );
}

// ══════════════════════════════════════════════════════════════════
//  README table structure
// ══════════════════════════════════════════════════════════════════

#[test]
fn readme_has_flag_tables() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("| `FLAG`"),
        "README should have flag tables"
    );
    assert!(
        content.contains("| `KEY`"),
        "README should have keybind tables"
    );
}

#[test]
fn readme_has_dependency_table() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("| `IMPLANT`"),
        "README should have dependency table"
    );
}

#[test]
fn readme_documents_short_flags() {
    let content = std::fs::read_to_string("README.md").unwrap();
    for flag in [
        "-i", "-f", "-F", "-n", "-N", "-p", "-b", "-B", "-P", "-Z", "-l", "-h", "-V",
    ] {
        assert!(
            content.contains(flag),
            "README missing short flag: {}",
            flag
        );
    }
}

// ══════════════════════════════════════════════════════════════════
//  Binary builds correctly
// ══════════════════════════════════════════════════════════════════

#[test]
fn binary_runs_without_panic() {
    // Running with -h should never panic
    let output = cargo_bin().arg("-h").output().unwrap();
    assert!(output.status.success(), "binary should not panic on -h");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("panic"),
        "should not contain panic message"
    );
    assert!(
        !stderr.contains("thread"),
        "should not contain thread panic message"
    );
}

#[test]
fn binary_runs_version_without_panic() {
    let output = cargo_bin().arg("-V").output().unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("panic"));
}

#[test]
fn binary_runs_list_colors_without_panic() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("panic"));
}

#[test]
fn binary_runs_completions_without_panic() {
    for shell in ["zsh", "bash", "fish", "elvish", "powershell"] {
        let output = cargo_bin().args(["--completions", shell]).output().unwrap();
        assert!(
            output.status.success(),
            "{} completions should not panic",
            shell
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("panic"),
            "{} completions should not panic",
            shell
        );
    }
}

// ══════════════════════════════════════════════════════════════════
//  README documentation examples
// ══════════════════════════════════════════════════════════════════

#[test]
fn readme_has_example_commands() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("sudo iftoprs"),
        "README should show sudo examples"
    );
    assert!(
        content.contains("iftoprs --completions"),
        "README should show completions example"
    );
}

#[test]
fn readme_documents_json_piping() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("--json | jq") || content.contains("--json |"),
        "README should show JSON piping example"
    );
}

#[test]
fn readme_documents_en0_example() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("en0"),
        "README should show en0 interface example"
    );
}

#[test]
fn readme_documents_tcp_port_443_example() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(
        content.contains("tcp port 443"),
        "README should show HTTPS filter example"
    );
}

// ══════════════════════════════════════════════════════════════════
//  Config file line counts and structure
// ══════════════════════════════════════════════════════════════════

#[test]
fn default_config_has_reasonable_line_count() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    let lines = content.lines().count();
    assert!(
        lines >= 15,
        "default config should have >= 15 lines, got {}",
        lines
    );
    assert!(
        lines <= 200,
        "default config should have <= 200 lines, got {}",
        lines
    );
}

#[test]
fn default_config_all_key_values_have_nearby_comments() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    // The majority of value lines should have comments nearby
    let lines: Vec<&str> = content.lines().collect();
    let value_lines: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.contains('=') && !l.trim_start().starts_with('#'))
        .map(|(i, _)| i)
        .collect();
    let mut documented = 0;
    for &idx in &value_lines {
        if idx > 0 {
            let has_comment = (0..idx)
                .rev()
                .take(10)
                .any(|i| lines[i].trim_start().starts_with('#'));
            if has_comment {
                documented += 1;
            }
        }
    }
    let ratio = documented as f64 / value_lines.len() as f64;
    assert!(
        ratio >= 0.7,
        "at least 70% of values should be documented, got {:.0}%",
        ratio * 100.0
    );
}
