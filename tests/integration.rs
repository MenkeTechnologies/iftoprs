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
