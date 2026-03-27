use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::theme::{CustomThemeColors, ThemeName};
use crate::ui::app::{BarStyle, PinnedFlow};

/// Persistent preferences saved to ~/.iftoprs.conf
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefs {
    #[serde(default)]
    pub theme: ThemeName,
    #[serde(default = "default_true")]
    pub dns_resolution: bool,
    #[serde(default = "default_true")]
    pub port_resolution: bool,
    #[serde(default = "default_true")]
    pub show_ports: bool,
    #[serde(default = "default_true")]
    pub show_bars: bool,
    #[serde(default)]
    pub use_bytes: bool,
    #[serde(default)]
    pub show_processes: bool,
    #[serde(default)]
    pub show_cumulative: bool,
    #[serde(default)]
    pub bar_style: BarStyle,
    #[serde(default)]
    pub pinned: Vec<PinnedFlow>,
    #[serde(default = "default_true")]
    pub show_border: bool,
    #[serde(default = "default_true")]
    pub show_header: bool,
    #[serde(default = "default_refresh")]
    pub refresh_rate: u64,
    #[serde(default)]
    pub alert_threshold: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_themes: HashMap<String, CustomThemeColors>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_custom_theme: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_refresh() -> u64 {
    1
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            theme: ThemeName::default(),
            dns_resolution: true,
            port_resolution: true,
            show_ports: true,
            show_bars: true,
            use_bytes: false,
            show_processes: true,
            show_cumulative: false,
            bar_style: BarStyle::default(),
            pinned: Vec::new(),
            show_border: true,
            show_header: true,
            refresh_rate: 1,
            alert_threshold: 0.0,
            interface: None,
            custom_themes: HashMap::new(),
            active_custom_theme: None,
        }
    }
}

use std::sync::OnceLock;

static CUSTOM_CONFIG_PATH: OnceLock<std::path::PathBuf> = OnceLock::new();

/// Set a custom config file path (call once at startup).
pub fn set_config_path(path: std::path::PathBuf) {
    let _ = CUSTOM_CONFIG_PATH.set(path);
}

fn prefs_path() -> Option<std::path::PathBuf> {
    if let Some(p) = CUSTOM_CONFIG_PATH.get() {
        return Some(p.clone());
    }
    dirs::home_dir().map(|h| h.join(".iftoprs.conf"))
}

pub fn load_prefs() -> Prefs {
    let path = match prefs_path() {
        Some(p) => p,
        None => return Prefs::default(),
    };
    match std::fs::read_to_string(&path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
        Err(_) => {
            // Config doesn't exist — write the default
            let prefs = Prefs::default();
            save_prefs(&prefs);
            prefs
        }
    }
}

pub fn save_prefs(prefs: &Prefs) {
    #[cfg(test)]
    { let _ = prefs; return; }

    #[cfg(not(test))]
    if let Some(path) = prefs_path()
        && let Ok(s) = toml::to_string_pretty(prefs) {
            let _ = std::fs::write(path, s);
        }
}
