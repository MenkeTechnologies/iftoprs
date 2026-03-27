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
