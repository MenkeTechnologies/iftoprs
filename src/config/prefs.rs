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
    #[serde(default = "default_true")]
    pub hover_tooltips: bool,
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
            hover_tooltips: true,
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
    {
        let _ = prefs;
    }

    #[cfg(not(test))]
    if let Some(path) = prefs_path()
        && let Ok(s) = toml::to_string_pretty(prefs)
    {
        let _ = std::fs::write(path, s);
    }
}

#[cfg(test)]
mod tests {
    use super::super::theme::CustomThemeColors;
    use super::*;

    #[test]
    fn prefs_default_values() {
        let p = Prefs::default();
        assert_eq!(p.theme, ThemeName::default());
        assert!(p.dns_resolution);
        assert!(p.port_resolution);
        assert!(p.show_ports);
        assert!(p.show_bars);
        assert!(!p.use_bytes);
        assert!(p.show_processes);
        assert!(!p.show_cumulative);
        assert_eq!(p.bar_style, BarStyle::default());
        assert!(p.pinned.is_empty());
        assert!(p.show_border);
        assert!(p.show_header);
        assert_eq!(p.refresh_rate, 1);
        assert_eq!(p.alert_threshold, 0.0);
        assert!(p.interface.is_none());
    }

    #[test]
    fn prefs_serialize_deserialize() {
        let p = Prefs::default();
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.theme, p.theme);
        assert_eq!(p2.dns_resolution, p.dns_resolution);
        assert_eq!(p2.show_border, p.show_border);
        assert_eq!(p2.refresh_rate, p.refresh_rate);
    }

    #[test]
    fn prefs_deserialize_empty_toml() {
        let p: Prefs = toml::from_str("").unwrap();
        assert_eq!(p.theme, ThemeName::default());
        assert!(p.show_border);
        assert_eq!(p.refresh_rate, 1);
    }

    #[test]
    fn prefs_deserialize_partial_toml() {
        let p: Prefs = toml::from_str("theme = \"BladeRunner\"\nuse_bytes = true").unwrap();
        assert_eq!(p.theme, ThemeName::BladeRunner);
        assert!(p.use_bytes);
        // defaults for missing fields
        assert!(p.show_border);
        assert_eq!(p.refresh_rate, 1);
    }

    #[test]
    fn prefs_interface_none_omitted() {
        let p = Prefs::default();
        let s = toml::to_string_pretty(&p).unwrap();
        assert!(!s.contains("interface"));
    }

    #[test]
    fn prefs_interface_some_included() {
        let p = Prefs {
            interface: Some("en0".into()),
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        assert!(s.contains("interface = \"en0\""));
    }

    #[test]
    fn prefs_interface_roundtrip() {
        let p = Prefs {
            interface: Some("eth0".into()),
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.interface, Some("eth0".into()));
    }

    #[test]
    fn prefs_pinned_roundtrip() {
        let mut p = Prefs::default();
        p.pinned.push(crate::ui::app::PinnedFlow {
            src: "10.0.0.1".into(),
            dst: "10.0.0.2".into(),
        });
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.pinned.len(), 1);
        assert_eq!(p2.pinned[0].src, "10.0.0.1");
    }

    #[test]
    fn prefs_custom_values_roundtrip() {
        let p = Prefs {
            theme: ThemeName::GlitchPop,
            use_bytes: true,
            show_border: false,
            refresh_rate: 5,
            alert_threshold: 1000.0,
            bar_style: BarStyle::Thin,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.theme, ThemeName::GlitchPop);
        assert!(p2.use_bytes);
        assert!(!p2.show_border);
        assert_eq!(p2.refresh_rate, 5);
        assert_eq!(p2.alert_threshold, 1000.0);
        assert_eq!(p2.bar_style, BarStyle::Thin);
    }

    #[test]
    fn save_prefs_no_op_in_test() {
        let p = Prefs::default();
        save_prefs(&p); // should not panic or write to disk
    }

    #[test]
    fn load_prefs_returns_valid() {
        let p = load_prefs();
        // Should always return a valid Prefs, regardless of file state
        assert!(p.refresh_rate >= 1);
    }

    #[test]
    fn default_true_helper() {
        assert!(default_true());
    }

    #[test]
    fn default_refresh_helper() {
        assert_eq!(default_refresh(), 1);
    }

    #[test]
    fn prefs_invalid_theme_string_yields_default_prefs() {
        let p: Prefs = toml::from_str(r#"theme = "TotallyUnknownThemeName""#).unwrap_or_default();
        assert_eq!(p.theme, ThemeName::default());
    }

    #[test]
    fn prefs_deserialize_invalid_toml_fails() {
        assert!(toml::from_str::<Prefs>("not valid toml {{{").is_err());
    }

    #[test]
    fn prefs_deserialize_wrong_type_for_refresh_rate_fails() {
        assert!(toml::from_str::<Prefs>(r#"refresh_rate = "five""#).is_err());
    }

    #[test]
    fn prefs_hover_tooltips_roundtrip() {
        let p = Prefs {
            hover_tooltips: false,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(!p2.hover_tooltips);
    }

    #[test]
    fn prefs_alert_threshold_roundtrip() {
        let p = Prefs {
            alert_threshold: 42.5,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!((p2.alert_threshold - 42.5).abs() < f64::EPSILON);
    }

    #[test]
    fn prefs_show_cumulative_roundtrip() {
        let p = Prefs {
            show_cumulative: true,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(p2.show_cumulative);
    }

    #[test]
    fn prefs_port_resolution_false_roundtrip() {
        let p = Prefs {
            port_resolution: false,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(!p2.port_resolution);
    }

    #[test]
    fn prefs_bar_style_ascii_roundtrip() {
        let p = Prefs {
            bar_style: crate::ui::app::BarStyle::Ascii,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.bar_style, crate::ui::app::BarStyle::Ascii);
    }

    #[test]
    fn prefs_dns_resolution_false_roundtrip() {
        let p = Prefs {
            dns_resolution: false,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(!p2.dns_resolution);
    }

    #[test]
    fn prefs_show_processes_false_roundtrip() {
        let p = Prefs {
            show_processes: false,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(!p2.show_processes);
    }

    #[test]
    fn prefs_show_header_false_roundtrip() {
        let p = Prefs {
            show_header: false,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(!p2.show_header);
    }

    #[test]
    fn prefs_hover_tooltips_false_roundtrip() {
        let p = Prefs {
            hover_tooltips: false,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(!p2.hover_tooltips);
    }

    #[test]
    fn prefs_show_border_false_roundtrip() {
        let p = Prefs {
            show_border: false,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(!p2.show_border);
    }

    #[test]
    fn prefs_custom_themes_and_active_roundtrip() {
        let mut p = Prefs::default();
        p.custom_themes.insert(
            "mine".into(),
            CustomThemeColors {
                c1: 10,
                c2: 20,
                c3: 30,
                c4: 40,
                c5: 50,
                c6: 60,
            },
        );
        p.active_custom_theme = Some("mine".into());
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.active_custom_theme.as_deref(), Some("mine"));
        let c = p2.custom_themes.get("mine").unwrap();
        assert_eq!((c.c1, c.c6), (10, 60));
    }

    #[test]
    fn prefs_show_ports_false_roundtrip() {
        let p = Prefs {
            show_ports: false,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(!p2.show_ports);
    }

    #[test]
    fn prefs_show_bars_false_roundtrip() {
        let p = Prefs {
            show_bars: false,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(!p2.show_bars);
    }

    #[test]
    fn prefs_theme_quantum_flux_roundtrip() {
        let p = Prefs {
            theme: ThemeName::QuantumFlux,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.theme, ThemeName::QuantumFlux);
    }

    #[test]
    fn prefs_use_bytes_true_roundtrip() {
        let p = Prefs {
            use_bytes: true,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(p2.use_bytes);
    }

    #[test]
    fn prefs_two_pinned_flows_roundtrip() {
        let mut p = Prefs::default();
        p.pinned.push(crate::ui::app::PinnedFlow {
            src: "192.0.2.1".into(),
            dst: "192.0.2.2".into(),
        });
        p.pinned.push(crate::ui::app::PinnedFlow {
            src: "2001:db8::1".into(),
            dst: "2001:db8::2".into(),
        });
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.pinned.len(), 2);
        assert_eq!(p2.pinned[1].src, "2001:db8::1");
    }

    #[test]
    fn prefs_theme_zaibatsu_roundtrip() {
        let p = Prefs {
            theme: ThemeName::Zaibatsu,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.theme, ThemeName::Zaibatsu);
    }

    #[test]
    fn prefs_theme_iftopcolor_roundtrip() {
        let p = Prefs {
            theme: ThemeName::Iftopcolor,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.theme, ThemeName::Iftopcolor);
    }

    #[test]
    fn prefs_theme_chrome_heart_roundtrip() {
        let p = Prefs {
            theme: ThemeName::ChromeHeart,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.theme, ThemeName::ChromeHeart);
    }

    #[test]
    fn prefs_interface_some_roundtrip() {
        let p = Prefs {
            interface: Some("en0".into()),
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.interface.as_deref(), Some("en0"));
    }

    #[test]
    fn prefs_theme_synth_wave_roundtrip() {
        let p = Prefs {
            theme: ThemeName::SynthWave,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.theme, ThemeName::SynthWave);
    }

    #[test]
    fn prefs_theme_plasma_core_roundtrip() {
        let p = Prefs {
            theme: ThemeName::PlasmaCore,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.theme, ThemeName::PlasmaCore);
    }

    #[test]
    fn prefs_theme_night_city_roundtrip() {
        let p = Prefs {
            theme: ThemeName::NightCity,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.theme, ThemeName::NightCity);
    }

    #[test]
    fn prefs_theme_void_walker_roundtrip() {
        let p = Prefs {
            theme: ThemeName::VoidWalker,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.theme, ThemeName::VoidWalker);
    }

    #[test]
    fn prefs_refresh_rate_ten_roundtrip() {
        let p = Prefs {
            refresh_rate: 10,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.refresh_rate, 10);
    }

    #[test]
    fn prefs_alert_threshold_negative_roundtrip() {
        let p = Prefs {
            alert_threshold: -1.0,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!((p2.alert_threshold + 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn prefs_theme_neon_noir_roundtrip() {
        let p = Prefs {
            theme: ThemeName::NeonNoir,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.theme, ThemeName::NeonNoir);
    }

    #[test]
    fn prefs_empty_pinned_array_roundtrip() {
        let p = Prefs {
            pinned: Vec::new(),
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(p2.pinned.is_empty());
    }

    #[test]
    fn prefs_interface_none_omitted_from_toml() {
        let p = Prefs::default();
        let s = toml::to_string_pretty(&p).unwrap();
        assert!(!s.contains("interface"));
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(p2.interface.is_none());
    }

    #[test]
    fn prefs_bar_style_gradient_roundtrip() {
        let p = Prefs {
            bar_style: BarStyle::Gradient,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.bar_style, BarStyle::Gradient);
    }

    #[test]
    fn prefs_bar_style_solid_roundtrip() {
        let p = Prefs {
            bar_style: BarStyle::Solid,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.bar_style, BarStyle::Solid);
    }

    #[test]
    fn prefs_active_custom_theme_none_omitted_from_toml() {
        let p = Prefs::default();
        let s = toml::to_string_pretty(&p).unwrap();
        assert!(!s.contains("active_custom_theme"));
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(p2.active_custom_theme.is_none());
    }

    #[test]
    fn prefs_use_bytes_true_and_show_ports_false_roundtrip() {
        let p = Prefs {
            use_bytes: true,
            show_ports: false,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(p2.use_bytes);
        assert!(!p2.show_ports);
    }

    #[test]
    fn prefs_refresh_rate_one_hour_roundtrip() {
        let p = Prefs {
            refresh_rate: 3600,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.refresh_rate, 3600);
    }

    #[test]
    fn prefs_show_header_false_show_border_true_roundtrip() {
        let p = Prefs {
            show_header: false,
            show_border: true,
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(!p2.show_header);
        assert!(p2.show_border);
    }

    #[test]
    fn prefs_custom_themes_empty_map_roundtrip() {
        let p = Prefs {
            custom_themes: std::collections::HashMap::new(),
            ..Default::default()
        };
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert!(p2.custom_themes.is_empty());
    }
}
