use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// All named color themes — ported from storageshower.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeName {
    #[default]
    NeonSprawl,
    AcidRain,
    IceBreaker,
    SynthWave,
    RustBelt,
    GhostWire,
    RedSector,
    SakuraDen,
    DataStream,
    SolarFlare,
    NeonNoir,
    ChromeHeart,
    BladeRunner,
    VoidWalker,
    ToxicWaste,
    CyberFrost,
    PlasmaCore,
    SteelNerve,
    DarkSignal,
    GlitchPop,
    HoloShift,
    NightCity,
    DeepNet,
    LaserGrid,
    QuantumFlux,
    BioHazard,
    Darkwave,
    Overlock,
    Megacorp,
    Zaibatsu,
    Iftopcolor,
}

impl ThemeName {
    pub const ALL: &'static [ThemeName] = &[
        ThemeName::NeonSprawl,
        ThemeName::AcidRain,
        ThemeName::IceBreaker,
        ThemeName::SynthWave,
        ThemeName::RustBelt,
        ThemeName::GhostWire,
        ThemeName::RedSector,
        ThemeName::SakuraDen,
        ThemeName::DataStream,
        ThemeName::SolarFlare,
        ThemeName::NeonNoir,
        ThemeName::ChromeHeart,
        ThemeName::BladeRunner,
        ThemeName::VoidWalker,
        ThemeName::ToxicWaste,
        ThemeName::CyberFrost,
        ThemeName::PlasmaCore,
        ThemeName::SteelNerve,
        ThemeName::DarkSignal,
        ThemeName::GlitchPop,
        ThemeName::HoloShift,
        ThemeName::NightCity,
        ThemeName::DeepNet,
        ThemeName::LaserGrid,
        ThemeName::QuantumFlux,
        ThemeName::BioHazard,
        ThemeName::Darkwave,
        ThemeName::Overlock,
        ThemeName::Megacorp,
        ThemeName::Zaibatsu,
        ThemeName::Iftopcolor,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            ThemeName::NeonSprawl => "Neon Sprawl",
            ThemeName::AcidRain => "Acid Rain",
            ThemeName::IceBreaker => "Ice Breaker",
            ThemeName::SynthWave => "Synth Wave",
            ThemeName::RustBelt => "Rust Belt",
            ThemeName::GhostWire => "Ghost Wire",
            ThemeName::RedSector => "Red Sector",
            ThemeName::SakuraDen => "Sakura Den",
            ThemeName::DataStream => "Data Stream",
            ThemeName::SolarFlare => "Solar Flare",
            ThemeName::NeonNoir => "Neon Noir",
            ThemeName::ChromeHeart => "Chrome Heart",
            ThemeName::BladeRunner => "Blade Runner",
            ThemeName::VoidWalker => "Void Walker",
            ThemeName::ToxicWaste => "Toxic Waste",
            ThemeName::CyberFrost => "Cyber Frost",
            ThemeName::PlasmaCore => "Plasma Core",
            ThemeName::SteelNerve => "Steel Nerve",
            ThemeName::DarkSignal => "Dark Signal",
            ThemeName::GlitchPop => "Glitch Pop",
            ThemeName::HoloShift => "Holo Shift",
            ThemeName::NightCity => "Night City",
            ThemeName::DeepNet => "Deep Net",
            ThemeName::LaserGrid => "Laser Grid",
            ThemeName::QuantumFlux => "Quantum Flux",
            ThemeName::BioHazard => "Bio Hazard",
            ThemeName::Darkwave => "Darkwave",
            ThemeName::Overlock => "Overlock",
            ThemeName::Megacorp => "Megacorp",
            ThemeName::Zaibatsu => "Zaibatsu",
            ThemeName::Iftopcolor => "iftopcolor",
        }
    }
}

/// 6-color palette from storageshower: (primary, accent, c3, c4, c5, c6)
fn palette(name: ThemeName) -> (u8, u8, u8, u8, u8, u8) {
    match name {
        ThemeName::NeonSprawl => (27, 48, 135, 141, 63, 99),
        ThemeName::AcidRain => (28, 46, 34, 40, 22, 35),
        ThemeName::IceBreaker => (19, 39, 25, 33, 21, 32),
        ThemeName::SynthWave => (91, 177, 128, 134, 93, 97),
        ThemeName::RustBelt => (172, 214, 178, 220, 166, 130),
        ThemeName::GhostWire => (37, 50, 44, 87, 30, 23),
        ThemeName::RedSector => (160, 203, 196, 210, 124, 88),
        ThemeName::SakuraDen => (175, 218, 182, 225, 169, 132),
        ThemeName::DataStream => (22, 46, 28, 119, 34, 22),
        ThemeName::SolarFlare => (202, 220, 196, 213, 160, 125),
        ThemeName::NeonNoir => (201, 231, 93, 219, 57, 53),
        ThemeName::ChromeHeart => (250, 255, 246, 253, 243, 239),
        ThemeName::BladeRunner => (208, 37, 166, 73, 130, 23),
        ThemeName::VoidWalker => (55, 99, 54, 141, 92, 17),
        ThemeName::ToxicWaste => (118, 190, 154, 226, 82, 58),
        ThemeName::CyberFrost => (159, 195, 153, 189, 111, 67),
        ThemeName::PlasmaCore => (199, 213, 163, 207, 126, 89),
        ThemeName::SteelNerve => (68, 110, 60, 146, 24, 236),
        ThemeName::DarkSignal => (30, 43, 23, 79, 29, 16),
        ThemeName::GlitchPop => (201, 51, 226, 47, 196, 21),
        ThemeName::HoloShift => (123, 219, 159, 183, 87, 133),
        ThemeName::NightCity => (214, 227, 209, 223, 172, 94),
        ThemeName::DeepNet => (19, 33, 17, 75, 26, 16),
        ThemeName::LaserGrid => (46, 201, 51, 226, 196, 21),
        ThemeName::QuantumFlux => (135, 75, 171, 111, 98, 61),
        ThemeName::BioHazard => (148, 184, 106, 192, 64, 22),
        ThemeName::Darkwave => (53, 140, 89, 176, 127, 52),
        ThemeName::Overlock => (196, 208, 160, 214, 124, 52),
        ThemeName::Megacorp => (252, 39, 245, 81, 242, 236),
        ThemeName::Zaibatsu => (167, 216, 131, 224, 95, 52),
        ThemeName::Iftopcolor => (21, 46, 28, 48, 33, 19),
    }
}

/// Complete theme for the iftoprs UI, derived from a 6-color palette.
#[derive(Debug, Clone)]
pub struct Theme {
    pub bar_color: Color,
    pub bar_color_mid: Color,
    pub bar_text: Color,
    pub host_src: Color,
    pub host_dst: Color,
    pub arrow: Color,
    pub rate_2s: Color,
    pub rate_10s: Color,
    pub rate_40s: Color,
    pub scale_label: Color,
    pub scale_line: Color,
    pub total_label: Color,
    pub cum_label: Color,
    pub peak_label: Color,
    pub proc_name: Color,
    pub help_bg: Color,
    pub help_border: Color,
    pub help_title: Color,
    pub help_section: Color,
    pub help_key: Color,
    pub help_val: Color,
    pub select_bg: Color,
}

/// Custom theme colors stored in config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomThemeColors {
    pub c1: u8,
    pub c2: u8,
    pub c3: u8,
    pub c4: u8,
    pub c5: u8,
    pub c6: u8,
}

impl Theme {
    pub fn from_name(name: ThemeName) -> Self {
        let (c1, c2, c3, c4, c5, c6) = palette(name);
        Self::from_palette_raw(c1, c2, c3, c4, c5, c6)
    }

    pub fn from_palette_raw(c1: u8, c2: u8, c3: u8, c4: u8, c5: u8, c6: u8) -> Self {
        Theme {
            bar_color: Color::Indexed(c6), // darkest — bar background
            bar_color_mid: Color::Indexed(Self::shift_color_lighter(c6)),
            bar_text: Color::Black,
            host_src: Color::Indexed(c2), // accent — bright
            host_dst: Color::Indexed(c2),
            arrow: Color::Indexed(c5),
            rate_2s: Color::Indexed(c2),  // accent
            rate_10s: Color::Indexed(c4), // mid
            rate_40s: Color::Indexed(c5), // dim
            scale_label: Color::Indexed(c2),
            scale_line: Color::Indexed(c6),
            total_label: Color::Indexed(c1), // primary
            cum_label: Color::Indexed(c2),
            peak_label: Color::Indexed(c4),
            proc_name: Color::Indexed(c3),
            help_bg: Color::Indexed(236),
            help_border: Color::Indexed(c1),
            help_title: Color::Indexed(c1),
            help_section: Color::Indexed(c6),
            help_key: Color::Indexed(c2),
            help_val: Color::Indexed(c4),
            select_bg: Color::Indexed(236),
        }
    }

    /// Shift an indexed 256-color one step lighter in the color cube or grayscale ramp.
    fn shift_color_lighter(c: u8) -> u8 {
        if c >= 232 {
            // Grayscale ramp (232..=255): bump up
            c + 2
        } else if c >= 16 {
            // 6x6x6 color cube (16..=231)
            let idx = c - 16;
            let b = idx % 6;
            let g = (idx / 6) % 6;
            let r = idx / 36;
            let r2 = (r + 1).min(5);
            let g2 = (g + 1).min(5);
            let b2 = (b + 1).min(5);
            16 + r2 * 36 + g2 * 6 + b2
        } else {
            // Basic 16 colors — bump to a lighter variant if possible
            (c + 8).min(15)
        }
    }

    /// Get the raw 6-color palette values for a built-in theme.
    pub fn palette_values(name: ThemeName) -> [u8; 6] {
        let (c1, c2, c3, c4, c5, c6) = palette(name);
        [c1, c2, c3, c4, c5, c6]
    }

    /// Generate a 6-cell color swatch string for theme preview.
    pub fn swatch(name: ThemeName) -> Vec<(Color, &'static str)> {
        let (c1, c2, c3, c4, c5, c6) = palette(name);
        vec![
            (Color::Indexed(c1), "██"),
            (Color::Indexed(c2), "██"),
            (Color::Indexed(c3), "██"),
            (Color::Indexed(c4), "██"),
            (Color::Indexed(c5), "██"),
            (Color::Indexed(c6), "██"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_themes_count() {
        assert_eq!(ThemeName::ALL.len(), 31);
    }

    #[test]
    fn default_theme_is_neon_sprawl() {
        assert_eq!(ThemeName::default(), ThemeName::NeonSprawl);
    }

    #[test]
    fn all_themes_have_display_names() {
        for &name in ThemeName::ALL {
            assert!(!name.display_name().is_empty());
        }
    }

    #[test]
    fn all_themes_produce_valid_theme() {
        for &name in ThemeName::ALL {
            let _theme = Theme::from_name(name);
        }
    }

    #[test]
    fn swatch_has_six_colors() {
        for &name in ThemeName::ALL {
            assert_eq!(Theme::swatch(name).len(), 6);
        }
    }

    #[test]
    fn theme_fields_are_indexed_colors() {
        let t = Theme::from_name(ThemeName::NeonSprawl);
        assert!(matches!(t.bar_color, Color::Indexed(_)));
        assert_eq!(t.bar_text, Color::Black);
        assert!(matches!(t.host_src, Color::Indexed(_)));
        assert!(matches!(t.arrow, Color::Indexed(_)));
        assert!(matches!(t.proc_name, Color::Indexed(_)));
        assert_eq!(t.help_bg, Color::Indexed(236));
    }

    #[test]
    fn all_themes_unique_display_names() {
        let mut names: Vec<&str> = ThemeName::ALL.iter().map(|t| t.display_name()).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), ThemeName::ALL.len());
    }

    #[test]
    fn neon_sprawl_palette() {
        let t = Theme::from_name(ThemeName::NeonSprawl);
        assert_eq!(t.bar_color, Color::Indexed(99));
        assert_eq!(t.host_src, Color::Indexed(48));
        assert_eq!(t.total_label, Color::Indexed(27));
        assert_eq!(t.proc_name, Color::Indexed(135));
    }

    #[test]
    fn blade_runner_palette() {
        let t = Theme::from_name(ThemeName::BladeRunner);
        assert_eq!(t.bar_color, Color::Indexed(23));
        assert_eq!(t.host_src, Color::Indexed(37));
        assert_eq!(t.total_label, Color::Indexed(208));
    }

    #[test]
    fn swatch_colors_match_palette() {
        let s = Theme::swatch(ThemeName::NeonSprawl);
        assert_eq!(s[0].0, Color::Indexed(27));
        assert_eq!(s[1].0, Color::Indexed(48));
        assert_eq!(s[5].0, Color::Indexed(99));
    }

    #[test]
    fn theme_name_serde_roundtrip() {
        let name = ThemeName::BladeRunner;
        let json = serde_json::to_string(&name).unwrap();
        let parsed: ThemeName = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, name);
    }

    #[test]
    fn theme_all_contains_default() {
        assert!(ThemeName::ALL.contains(&ThemeName::default()));
    }

    #[test]
    fn theme_clone() {
        let t = Theme::from_name(ThemeName::AcidRain);
        let t2 = t.clone();
        assert_eq!(t.bar_color, t2.bar_color);
        assert_eq!(t.host_src, t2.host_src);
    }

    #[test]
    fn theme_name_json_roundtrip_all_variants() {
        for &name in ThemeName::ALL {
            let json = serde_json::to_string(&name).unwrap();
            let back: ThemeName = serde_json::from_str(&json).unwrap();
            assert_eq!(back, name);
        }
    }

    #[test]
    fn theme_from_name_zaibatsu_has_indexed_colors() {
        let t = Theme::from_name(ThemeName::Zaibatsu);
        assert!(matches!(t.bar_color, Color::Indexed(_)));
    }

    #[test]
    fn display_name_never_equals_debug_string() {
        let n = ThemeName::NeonSprawl;
        assert_ne!(n.display_name(), format!("{:?}", n));
    }

    #[test]
    fn iftopcolor_theme_renders() {
        let t = Theme::from_name(ThemeName::Iftopcolor);
        assert!(matches!(t.bar_color, Color::Indexed(_)));
    }

    #[test]
    fn megacorp_theme_swatch_len() {
        assert_eq!(Theme::swatch(ThemeName::Megacorp).len(), 6);
    }

    #[test]
    fn every_theme_name_debug_is_single_token() {
        for &name in ThemeName::ALL {
            let d = format!("{:?}", name);
            assert!(!d.contains(' '), "{name:?}");
        }
    }

    #[test]
    fn zaibatsu_from_name_has_indexed_host_colors() {
        let t = Theme::from_name(ThemeName::Zaibatsu);
        assert!(matches!(t.host_src, Color::Indexed(_)));
        assert!(matches!(t.host_dst, Color::Indexed(_)));
    }

    #[test]
    fn palette_values_matches_swatch_indexed_colors() {
        for &name in ThemeName::ALL {
            let pal = Theme::palette_values(name);
            let sw = Theme::swatch(name);
            for i in 0..6 {
                assert_eq!(Color::Indexed(pal[i]), sw[i].0);
            }
        }
    }

    #[test]
    fn from_palette_raw_zero_grayscale_bar_mid() {
        let t = Theme::from_palette_raw(1, 2, 3, 4, 5, 240);
        assert!(matches!(t.bar_color, Color::Indexed(240)));
        assert!(matches!(t.bar_color_mid, Color::Indexed(_)));
    }

    #[test]
    fn palette_values_always_six_bytes_per_theme() {
        for &name in ThemeName::ALL {
            assert_eq!(Theme::palette_values(name).len(), 6);
        }
    }
}
