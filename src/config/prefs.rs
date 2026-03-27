use serde::{Deserialize, Serialize};

use super::theme::ThemeName;
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
}

fn default_true() -> bool {
    true
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
        }
    }
}

fn prefs_path() -> Option<std::path::PathBuf> {
    dirs::home_dir().map(|h| h.join(".iftoprs.conf"))
}

pub fn load_prefs() -> Prefs {
    let path = match prefs_path() {
        Some(p) => p,
        None => return Prefs::default(),
    };
    match std::fs::read_to_string(&path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
        Err(_) => Prefs::default(),
    }
}

pub fn save_prefs(prefs: &Prefs) {
    if let Some(path) = prefs_path()
        && let Ok(s) = toml::to_string_pretty(prefs) {
            let _ = std::fs::write(path, s);
        }
}
