use std::process::Command;

fn cargo_bin() -> Command {
    let mut cmd = Command::new(env!("CARGO"));
    cmd.args(["run", "--quiet", "--"]);
    cmd
}

#[test]
fn help_flag_shows_banner() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BANDWIDTH MONITOR"), "help should contain banner tagline");
    assert!(stdout.contains("USAGE"), "help should contain USAGE");
    assert!(stdout.contains("--interface"), "help should list --interface flag");
    assert!(stdout.contains("--no-dns"), "help should list --no-dns flag");
    assert!(stdout.contains("KEYBINDS"), "help should contain KEYBINDS section");
}

#[test]
fn version_flag_shows_version() {
    let output = cargo_bin().arg("-V").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.starts_with("iftoprs "), "should start with 'iftoprs '");
    assert!(stdout.contains('.'), "version should contain a dot");
}

#[test]
fn completions_zsh_generates_valid_output() {
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("#compdef iftoprs"), "should contain compdef header");
    assert!(stdout.contains("--interface"), "completions should include --interface");
    assert!(stdout.contains("--no-dns"), "completions should include --no-dns");
    assert!(stdout.contains("--completions"), "completions should include --completions");
    assert!(stdout.contains("--no-processes"), "completions should include --no-processes");
}

#[test]
fn completions_bash_generates_valid_output() {
    let output = cargo_bin().args(["--completions", "bash"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("_iftoprs"), "should contain completion function");
    assert!(stdout.contains("COMPREPLY"), "should contain COMPREPLY");
}

#[test]
fn help_contains_all_flags() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let flags = [
        "--interface", "--filter", "--net-filter", "--no-dns",
        "--no-port-names", "--promiscuous", "--no-bars", "--bytes",
        "--hide-ports", "--no-processes", "--list-interfaces",
    ];
    for flag in &flags {
        assert!(stdout.contains(flag), "help missing flag: {}", flag);
    }
}

#[test]
fn help_contains_ansi_colors() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\x1b["), "help should contain ANSI escape codes");
}

#[test]
fn help_contains_new_keybinds() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("scroll"), "help should document scroll keybinds");
    assert!(stdout.contains("disconnect"), "help should document quit keybind");
}

#[test]
fn help_shows_system_section() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("SYSTEM"), "help should contain SYSTEM section");
    assert!(stdout.contains("MenkeTechnologies"), "help should credit author");
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
        assert!(part.parse::<u32>().is_ok(), "non-numeric version part: {}", part);
    }
}

#[test]
fn help_contains_border_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("border"), "help should document border toggle");
}

#[test]
fn help_contains_filter_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("filter"), "help should document filter keybind");
}

#[test]
fn help_contains_theme_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("themes"), "help should document theme keybind");
}

#[test]
fn help_contains_pause_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pause"), "help should document pause keybind");
}

#[test]
fn list_colors_shows_all_themes() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Neon Sprawl"), "should list Neon Sprawl theme");
    assert!(stdout.contains("Blade Runner"), "should list Blade Runner theme");
    assert!(stdout.contains("iftopcolor"), "should list iftopcolor theme");
}

#[test]
fn default_config_file_exists() {
    let path = std::path::Path::new("iftoprs.default.conf");
    assert!(path.exists(), "iftoprs.default.conf should exist in project root");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("theme"), "default config should contain theme");
    assert!(content.contains("show_border"), "default config should contain show_border");
    assert!(content.contains("refresh_rate"), "default config should contain refresh_rate");
    assert!(content.contains("alert_threshold"), "default config should contain alert_threshold");
    assert!(content.contains("pinned"), "default config should contain pinned");
}

#[test]
fn completions_zsh_includes_list_colors() {
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--list-colors"), "zsh completions should include --list-colors");
}

#[test]
fn default_config_has_interface_docs() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("interface"), "default config should document interface field");
}

#[test]
fn help_contains_interface_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // -h help mentions interface flag
    assert!(stdout.contains("--interface"), "help should show --interface flag");
}

#[test]
fn help_contains_config_flag() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--config"), "help should show --config flag");
    assert!(stdout.contains("-c"), "help should show -c short flag");
}

#[test]
fn completions_fish_generates_valid_output() {
    let output = cargo_bin().args(["--completions", "fish"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("iftoprs"), "fish completions should reference iftoprs");
    assert!(stdout.contains("interface"), "fish completions should include interface");
}

#[test]
fn completions_zsh_includes_config_flag() {
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--config"), "zsh completions should include --config");
}

#[test]
fn completions_bash_includes_config_flag() {
    let output = cargo_bin().args(["--completions", "bash"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("config"), "bash completions should include config");
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
    assert!(output.status.success(), "--list-colors should exit with code 0");
}

#[test]
fn completions_exit_code_zero() {
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    assert!(output.status.success(), "--completions zsh should exit with code 0");
}

#[test]
fn help_banner_has_ascii_art() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("██"), "help banner should have block characters");
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
    assert!(stdout.contains("Usage"), "list-colors should show usage hint");
    assert!(stdout.contains("Cycle"), "list-colors should show cycle hint");
}

#[test]
fn help_contains_header_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("header"), "help should document header toggle");
}

#[test]
fn help_contains_refresh_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("refresh"), "help should document refresh rate keybind");
}

#[test]
fn help_contains_sort_keybinds() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sort"), "help should document sort keybinds");
    assert!(stdout.contains("freeze"), "help should document freeze order keybind");
}

#[test]
fn help_contains_processes_flag() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--no-processes"), "help should show --no-processes");
    assert!(stdout.contains("-Z"), "help should show -Z short flag");
}

#[test]
fn default_config_has_all_fields() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    let expected_fields = [
        "theme", "show_border", "show_ports", "show_bars",
        "show_processes", "show_header", "refresh_rate",
        "alert_threshold", "pinned",
    ];
    for field in &expected_fields {
        assert!(content.contains(field), "default config missing field: {}", field);
    }
}

#[test]
fn help_mentions_capture_section() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("CAPTURE"), "help should have CAPTURE section");
}

#[test]
fn help_contains_json_flag() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--json"), "help should show --json flag");
    assert!(stdout.contains("NDJSON"), "help should describe NDJSON output");
}

#[test]
fn help_contains_tab_keybind() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Tab"), "help should document Tab key for switching views");
    assert!(stdout.contains("switch view"), "help should explain Tab switches views");
}

#[test]
fn completions_zsh_includes_json_flag() {
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--json"), "zsh completions should include --json");
}

// ── Exit codes ──

#[test]
fn completions_bash_exit_code_zero() {
    let output = cargo_bin().args(["--completions", "bash"]).output().unwrap();
    assert!(output.status.success());
}

#[test]
fn completions_fish_exit_code_zero() {
    let output = cargo_bin().args(["--completions", "fish"]).output().unwrap();
    assert!(output.status.success());
}

#[test]
fn completions_elvish_exit_code_zero() {
    let output = cargo_bin().args(["--completions", "elvish"]).output().unwrap();
    assert!(output.status.success());
}

#[test]
fn completions_powershell_exit_code_zero() {
    let output = cargo_bin().args(["--completions", "powershell"]).output().unwrap();
    assert!(output.status.success());
}

// ── Completions content for all shells ──

#[test]
fn completions_elvish_generates_valid_output() {
    let output = cargo_bin().args(["--completions", "elvish"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "elvish completions should not be empty");
    assert!(stdout.contains("iftoprs"), "elvish completions should reference iftoprs");
}

#[test]
fn completions_powershell_generates_valid_output() {
    let output = cargo_bin().args(["--completions", "powershell"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "powershell completions should not be empty");
    assert!(stdout.contains("iftoprs"), "powershell completions should reference iftoprs");
}

// ── Completions include all flags ──

#[test]
fn completions_zsh_includes_all_flags() {
    let output = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for flag in ["--interface", "--filter", "--net-filter", "--no-dns", "--no-port-names",
                 "--promiscuous", "--no-bars", "--bytes", "--hide-ports", "--no-processes",
                 "--json", "--list-interfaces", "--list-colors", "--config", "--help", "--version"] {
        assert!(stdout.contains(flag), "zsh completions missing flag: {}", flag);
    }
}

#[test]
fn completions_bash_includes_all_flags() {
    let output = cargo_bin().args(["--completions", "bash"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for flag in ["interface", "filter", "no-dns", "no-bars", "bytes",
                 "hide-ports", "no-processes", "json", "list-colors"] {
        assert!(stdout.contains(flag), "bash completions missing flag: {}", flag);
    }
}

#[test]
fn completions_fish_includes_all_flags() {
    let output = cargo_bin().args(["--completions", "fish"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for flag in ["interface", "filter", "no-dns", "bytes", "json", "list-colors"] {
        assert!(stdout.contains(flag), "fish completions missing flag: {}", flag);
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
    for flag in ["-i", "-f", "-F", "-n", "-N", "-p", "-b", "-B", "-P", "-Z", "-l", "-h", "-V", "-c"] {
        assert!(stdout.contains(flag), "help missing short flag: {}", flag);
    }
}

#[test]
fn help_contains_bpf_example() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tcp port 80"), "help should show BPF filter example");
}

#[test]
fn help_contains_cidr_example() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("192.168.1.0/24"), "help should show CIDR example");
}

#[test]
fn help_contains_promiscuous_description() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("promiscuous"), "help should describe promiscuous mode");
}

// ── Keybind documentation ──

#[test]
fn help_documents_all_keybinds() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let keybinds = [
        "help HUD", "toggle DNS", "bars", "bytes/bits", "ports", "processes",
        "line mode", "cumulative", "pause", "border", "themes", "filter",
        "header bar", "refresh rate", "switch view",
        "sort by", "freeze order", "scroll", "disconnect",
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
    assert!(stdout.contains("JACK IN"), "help should show cyberpunk tagline");
    assert!(stdout.contains("neon rain"), "help should show neon rain quote");
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
    assert!(h_str.contains(version), "help banner should show same version as -V");
}

// ── List colors output ──

#[test]
fn list_colors_contains_ansi() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\x1b["), "list-colors should contain ANSI escape codes");
}

#[test]
fn list_colors_shows_all_31_themes() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let themes = [
        "Neon Sprawl", "Acid Rain", "Ice Breaker", "Synth Wave", "Rust Belt",
        "Ghost Wire", "Red Sector", "Sakura Den", "Data Stream", "Solar Flare",
        "Neon Noir", "Chrome Heart", "Blade Runner", "Void Walker", "Toxic Waste",
        "Cyber Frost", "Plasma Core", "Steel Nerve", "Dark Signal", "Glitch Pop",
        "Holo Shift", "Night City", "Deep Net", "Laser Grid", "Quantum Flux",
        "Bio Hazard", "Darkwave", "Overlock", "Megacorp", "Zaibatsu", "iftopcolor",
    ];
    for theme in &themes {
        assert!(stdout.contains(theme), "list-colors missing theme: {}", theme);
    }
}

#[test]
fn list_colors_shows_color_swatches() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Color swatches use 48;5;N escape codes for background
    assert!(stdout.contains("48;5;"), "list-colors should contain 256-color escapes");
}

#[test]
fn list_colors_has_section_header() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BUILTIN COLOR SCHEMES"), "should have section header");
}

// ── Default config file ──

#[test]
fn default_config_is_valid_toml() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    // Filter out comment lines and parse as TOML
    let filtered: String = content.lines()
        .filter(|l| !l.trim_start().starts_with('#') || l.contains('='))
        .collect::<Vec<_>>().join("\n");
    let parsed: toml::Value = toml::from_str(&filtered).unwrap();
    assert!(parsed.is_table());
}

#[test]
fn default_config_theme_is_neon_sprawl() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("theme = \"NeonSprawl\""), "default theme should be NeonSprawl");
}

#[test]
fn default_config_dns_enabled() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("dns_resolution = true"), "dns should be enabled by default");
}

#[test]
fn default_config_refresh_rate_one() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("refresh_rate = 1"), "default refresh rate should be 1");
}

#[test]
fn default_config_alert_disabled() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("alert_threshold = 0.0"), "alerts should be disabled by default");
}

#[test]
fn default_config_has_comments() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    let comment_count = content.lines().filter(|l| l.trim_start().starts_with('#')).count();
    assert!(comment_count >= 5, "default config should have documentation comments");
}

#[test]
fn default_config_documents_bar_styles() {
    let path = std::path::Path::new("iftoprs.default.conf");
    let content = std::fs::read_to_string(path).unwrap();
    assert!(content.contains("Gradient"), "should document Gradient bar style");
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
    assert!(content.starts_with("#compdef iftoprs"), "should start with #compdef");
}

#[test]
fn zsh_completion_file_has_function() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    assert!(content.contains("_iftoprs()"), "should define _iftoprs function");
}

#[test]
fn zsh_completion_file_includes_all_flags() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    for flag in ["--config", "--interface", "--filter", "--net-filter", "--no-dns",
                 "--no-port-names", "--promiscuous", "--no-bars", "--bytes",
                 "--hide-ports", "--no-processes", "--json", "--list-interfaces",
                 "--list-colors", "--help", "--version", "--completions"] {
        assert!(content.contains(flag), "_iftoprs missing flag: {}", flag);
    }
}

#[test]
fn zsh_completion_file_includes_short_flags() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    // Flags with values use '-X+[' or '-X[', boolean flags use '-X['
    for flag in ["-i", "-f", "-F", "-c"] {
        assert!(content.contains(&format!("'{flag}+")), "missing short flag with value: {}", flag);
    }
    for flag in ["-n", "-N", "-p", "-b", "-B", "-P", "-Z", "-l", "-h", "-V"] {
        assert!(content.contains(&format!("'{flag}[")), "missing short boolean flag: {}", flag);
    }
}

#[test]
fn zsh_completion_file_has_shell_completions_values() {
    let content = std::fs::read_to_string("completions/_iftoprs").unwrap();
    assert!(content.contains("bash"), "completions should list bash shell");
    assert!(content.contains("zsh"), "completions should list zsh shell");
    assert!(content.contains("fish"), "completions should list fish shell");
}

// ── Stderr output ──

#[test]
fn help_stderr_is_empty() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty(), "help should not write to stderr: {}", stderr);
}

#[test]
fn version_stderr_is_empty() {
    let output = cargo_bin().arg("-V").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty(), "version should not write to stderr: {}", stderr);
}

#[test]
fn list_colors_stderr_is_empty() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.is_empty(), "list-colors should not write to stderr: {}", stderr);
}

// ── Custom config flag ──

#[test]
fn config_flag_with_nonexistent_file_shows_help() {
    // -c takes a path argument, then -h is parsed as a separate flag
    let output = cargo_bin().args(["-c", "/tmp/nonexistent_iftoprs_test_12345.conf", "-h"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should show help output (the -h flag takes priority over needing a valid config)
    assert!(stdout.contains("BANDWIDTH MONITOR") || stdout.contains("IFTOPRS"),
        "should still show help with nonexistent config");
}

// ── Output consistency ──

#[test]
fn help_output_is_deterministic() {
    let out1 = cargo_bin().arg("-h").output().unwrap();
    let out2 = cargo_bin().arg("-h").output().unwrap();
    assert_eq!(out1.stdout, out2.stdout, "help output should be deterministic");
}

#[test]
fn version_output_is_deterministic() {
    let out1 = cargo_bin().arg("-V").output().unwrap();
    let out2 = cargo_bin().arg("-V").output().unwrap();
    assert_eq!(out1.stdout, out2.stdout, "version output should be deterministic");
}

#[test]
fn completions_output_is_deterministic() {
    let out1 = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let out2 = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    assert_eq!(out1.stdout, out2.stdout, "completions output should be deterministic");
}

// ── Help output size ──

#[test]
fn help_output_is_substantial() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.len() > 1000, "help should be at least 1000 bytes, got {}", stdout.len());
    let lines = stdout.lines().count();
    assert!(lines >= 30, "help should have at least 30 lines, got {}", lines);
}

#[test]
fn list_colors_output_is_substantial() {
    let output = cargo_bin().arg("--list-colors").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines = stdout.lines().count();
    assert!(lines >= 33, "list-colors should have at least 33 lines (31 themes + header/footer), got {}", lines);
}

// ── Completions generated match static file ──

#[test]
fn generated_zsh_completions_match_static_file() {
    let generated = cargo_bin().args(["--completions", "zsh"]).output().unwrap();
    let gen_str = String::from_utf8_lossy(&generated.stdout);
    let static_file = std::fs::read_to_string("completions/_iftoprs").unwrap();
    assert_eq!(gen_str.trim(), static_file.trim(),
        "generated zsh completions should match completions/_iftoprs");
}

// ── Invalid argument handling ──

#[test]
fn invalid_completions_shell_fails() {
    let output = cargo_bin().args(["--completions", "invalid_shell"]).output().unwrap();
    assert!(!output.status.success(), "invalid shell name should fail");
}

// ── Help banner structure ──

#[test]
fn help_banner_has_signal_bar() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("SIGNAL"), "banner should have SIGNAL indicator");
    assert!(stdout.contains("ONLINE"), "banner should show ONLINE status");
}

#[test]
fn help_shows_iftop_clone_description() {
    let output = cargo_bin().arg("-h").output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("iftop clone"), "should describe as iftop clone");
    assert!(stdout.contains("Rust"), "should mention Rust");
}

// ── Config file documentation ──

#[test]
fn default_config_lists_all_theme_names() {
    let content = std::fs::read_to_string("iftoprs.default.conf").unwrap();
    for theme in ["NeonSprawl", "BladeRunner", "Iftopcolor", "GlitchPop", "Zaibatsu"] {
        assert!(content.contains(theme), "default config should list theme: {}", theme);
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
    assert!(stdout.contains("NDJSON"), "help should describe NDJSON output");
    assert!(stdout.contains("no TUI"), "help should mention no TUI");
}

#[test]
fn completions_bash_includes_json() {
    let output = cargo_bin().args(["--completions", "bash"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("json"), "bash completions should include json");
}

#[test]
fn completions_fish_includes_json() {
    let output = cargo_bin().args(["--completions", "fish"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("json"), "fish completions should include json");
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
    assert!(stdout.contains("switch view"), "help should describe Tab as switching views");
}

// ── Cargo.toml dependencies ──

#[test]
fn cargo_toml_has_serde_json() {
    let content = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(content.contains("serde_json"), "Cargo.toml should include serde_json dependency");
}

// ── README features ──

#[test]
fn readme_documents_json_streaming() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(content.contains("--json"), "README should document --json flag");
    assert!(content.contains("NDJSON"), "README should mention NDJSON");
}

#[test]
fn readme_documents_process_view() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(content.contains("Tab"), "README should document Tab key");
    assert!(content.contains("process"), "README should mention process aggregation");
}

#[test]
fn readme_documents_jq_example() {
    let content = std::fs::read_to_string("README.md").unwrap();
    assert!(content.contains("jq"), "README should show jq piping example");
}
