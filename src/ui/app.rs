use std::collections::HashSet;
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::config::prefs::{self, Prefs};
use crate::config::theme::{CustomThemeColors, Theme, ThemeName};
use crate::data::flow::{FlowKey, Protocol};
use crate::data::tracker::{FlowSnapshot, TotalStats};
use crate::util::resolver::Resolver;

/// Tracks which fields were overridden by CLI flags (never saved to config).
#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    pub dns: bool,
    pub show_ports: bool,
    pub show_bars: bool,
    pub use_bytes: bool,
    pub show_processes: bool,
    pub interface: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewTab { Flows, Processes }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Avg2s, Avg10s, Avg40s, SrcName, DstName,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineDisplay { TwoLine, OneLine, SentOnly, RecvOnly }

impl LineDisplay {
    pub fn next(self) -> Self {
        match self {
            Self::TwoLine => Self::OneLine, Self::OneLine => Self::SentOnly,
            Self::SentOnly => Self::RecvOnly, Self::RecvOnly => Self::TwoLine,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BarStyle { #[default] Gradient, Solid, Thin, Ascii }

impl BarStyle {
    pub fn next(self) -> Self {
        match self {
            Self::Gradient => Self::Solid, Self::Solid => Self::Thin,
            Self::Thin => Self::Ascii, Self::Ascii => Self::Gradient,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Gradient => "gradient", Self::Solid => "solid",
            Self::Thin => "thin", Self::Ascii => "ascii",
        }
    }
}

/// Theme chooser popup state.
pub struct ThemeChooser {
    pub active: bool,
    pub selected: usize,
}

impl Default for ThemeChooser {
    fn default() -> Self {
        Self::new()
    }
}

impl ThemeChooser {
    pub fn new() -> Self { Self { active: false, selected: 0 } }
    pub fn open(&mut self, current: ThemeName) {
        self.active = true;
        self.selected = ThemeName::ALL.iter().position(|&t| t == current).unwrap_or(0);
    }
}

/// Interface chooser popup state.
pub struct InterfaceChooser {
    pub active: bool,
    pub selected: usize,
    pub interfaces: Vec<String>,
}

impl InterfaceChooser {
    pub fn new() -> Self { Self { active: false, selected: 0, interfaces: Vec::new() } }
    pub fn open(&mut self, current: &str) {
        self.interfaces = crate::capture::sniffer::list_interfaces().unwrap_or_default();
        if self.interfaces.is_empty() { return; }
        self.active = true;
        self.selected = self.interfaces.iter().position(|i| i == current).unwrap_or(0);
    }
}

impl Default for InterfaceChooser {
    fn default() -> Self { Self::new() }
}

/// Filter input state (/ key).
pub struct FilterState {
    pub active: bool,
    pub buf: String,
    pub cursor: usize,
    pub prev: Option<String>,
}

impl Default for FilterState {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterState {
    pub fn new() -> Self { Self { active: false, buf: String::new(), cursor: 0, prev: None } }
    pub fn open(&mut self, current: &Option<String>) {
        self.active = true;
        self.buf = current.clone().unwrap_or_default();
        self.cursor = self.buf.len();
        self.prev = current.clone();
    }
    pub fn insert(&mut self, ch: char) {
        self.buf.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            let prev = self.buf[..self.cursor].char_indices().next_back().map(|(i, _)| i).unwrap_or(0);
            self.buf.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }
    pub fn delete_word(&mut self) {
        // Ctrl+W behavior: skip trailing spaces, then delete the word
        let s = &self.buf[..self.cursor];
        let trimmed = s.trim_end();
        let word_start = trimmed.rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);
        self.buf.drain(word_start..self.cursor);
        self.cursor = word_start;
    }
    pub fn home(&mut self) { self.cursor = 0; }
    pub fn end(&mut self) { self.cursor = self.buf.len(); }
    pub fn left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.buf[..self.cursor].char_indices().next_back().map(|(i, _)| i).unwrap_or(0);
        }
    }
    pub fn right(&mut self) {
        if self.cursor < self.buf.len() {
            self.cursor = self.buf[self.cursor..].char_indices().nth(1).map(|(i, _)| self.cursor + i).unwrap_or(self.buf.len());
        }
    }
    pub fn kill_to_end(&mut self) { self.buf.truncate(self.cursor); }
}

/// Theme editor popup state.
#[derive(Default)]
pub struct ThemeEditState {
    pub active: bool,
    pub colors: [u8; 6],
    pub slot: usize,
    pub naming: bool,
    pub name: String,
    pub cursor: usize,
}

impl ThemeEditState {
    pub fn new() -> Self { Self::default() }
    pub fn open(&mut self, current_palette: [u8; 6]) {
        self.active = true;
        self.colors = current_palette;
        self.slot = 0;
        self.naming = false;
        self.name.clear();
        self.cursor = 0;
    }
}

/// Status message with auto-dismiss.
pub struct StatusMsg { pub text: String, pub since: Instant }

impl StatusMsg {
    pub fn new(text: String) -> Self { Self { text, since: Instant::now() } }
    pub fn expired(&self) -> bool { self.since.elapsed().as_secs() >= 3 }
}

/// Right-click tooltip state.
pub struct Tooltip {
    pub active: bool,
    pub x: u16,
    pub y: u16,
    pub lines: Vec<(String, String)>, // (label, value) pairs
}

impl Default for Tooltip {
    fn default() -> Self { Self::new() }
}

impl Tooltip {
    pub fn new() -> Self { Self { active: false, x: 0, y: 0, lines: Vec::new() } }
}

/// Hover state for timed tooltips on header bar segments.
#[derive(Default)]
pub struct HoverState {
    pub pos: Option<(u16, u16)>,
    pub since: Option<Instant>,
    pub right_click: bool,
}

impl HoverState {
    /// Returns true when hover has been active long enough to show tooltip (1s delay, 3s visible).
    pub fn ready(&self) -> bool {
        self.since
            .map(|t| {
                let elapsed = t.elapsed().as_millis();
                let visible = elapsed >= 1000;
                let expired = !self.right_click && elapsed >= 4000;
                visible && !expired
            })
            .unwrap_or(false)
    }

    /// Update position. Resets timer if position changed.
    pub fn move_to(&mut self, x: u16, y: u16) {
        let new_pos = (x, y);
        if self.pos != Some(new_pos) {
            self.pos = Some(new_pos);
            self.since = Some(Instant::now());
            self.right_click = false;
        }
    }

    /// Instant activation via right-click (bypasses 1s delay).
    pub fn right_click_at(&mut self, x: u16, y: u16) {
        self.pos = Some((x, y));
        self.since = Some(Instant::now() - std::time::Duration::from_secs(2));
        self.right_click = true;
    }

}

/// Alert state for bandwidth threshold crossing.
#[derive(Default)]
pub struct AlertState {
    /// Flow keys currently above threshold.
    pub alert_flows: HashSet<String>,
    /// When the last alert was triggered (for flash animation).
    pub flash: Option<Instant>,
}


impl AlertState {
    pub fn is_flashing(&self) -> bool {
        self.flash
            .map(|t| t.elapsed().as_millis() < 2000 && (t.elapsed().as_millis() / 300) % 2 == 0)
            .unwrap_or(false)
    }
}

/// A flow identity for pinning (favorites).
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PinnedFlow {
    pub src: String,
    pub dst: String,
}

/// Aggregated bandwidth data for a single process.
#[derive(Debug, Clone)]
pub struct ProcessSnapshot {
    pub name: String,
    pub pid: Option<u32>,
    pub flow_count: usize,
    pub sent_2s: f64,
    pub sent_10s: f64,
    pub sent_40s: f64,
    pub recv_2s: f64,
    pub recv_10s: f64,
    pub recv_40s: f64,
    pub total_sent: u64,
    pub total_recv: u64,
}

/// Application state for the TUI.
pub struct AppState {
    pub show_dns: bool,
    pub show_port_names: bool,
    pub show_ports: bool,
    pub show_bars: bool,
    pub show_cumulative: bool,
    pub show_processes: bool,
    pub use_bytes: bool,
    pub sort_column: SortColumn,
    pub sort_reverse: bool,
    pub line_display: LineDisplay,
    pub bar_style: BarStyle,
    pub paused: bool,
    pub scroll_offset: usize,
    pub selected: Option<usize>,
    pub show_help: bool,
    pub screen_filter: Option<String>,
    pub frozen_order: bool,
    pub theme_name: ThemeName,
    pub theme: Theme,
    pub theme_chooser: ThemeChooser,
    pub theme_edit: ThemeEditState,
    pub filter_state: FilterState,
    pub interface_chooser: InterfaceChooser,
    pub status_msg: Option<StatusMsg>,
    pub pinned: Vec<PinnedFlow>,
    pub tooltip: Tooltip,
    pub show_border: bool,
    pub show_header: bool,
    /// Y offset where the flow area starts (set by renderer).
    pub flow_area_y: u16,
    /// Y offset of the header bar (set by renderer).
    pub header_bar_y: u16,
    /// Hover state for timed tooltips.
    pub hover: HoverState,

    /// Alert system
    pub alert_state: AlertState,
    pub alert_threshold: f64,

    /// Refresh rate in seconds (1/2/5/10)
    pub refresh_rate: u64,

    /// Interface name for header display
    pub interface_name: String,
    /// Original interface from config (preserved across saves)
    pub config_interface: Option<String>,

    /// Custom themes created by user
    pub custom_themes: std::collections::HashMap<String, CustomThemeColors>,
    /// Currently active custom theme name (if any)
    pub active_custom_theme: Option<String>,

    /// Original prefs loaded from config (for fields overridden by CLI)
    pub orig_prefs: Prefs,
    /// Which fields were overridden by CLI flags
    pub cli_overrides: CliOverrides,

    /// Active view tab (Flows or Processes)
    pub view_tab: ViewTab,

    /// Total flow count before filtering (for header display)
    pub total_flow_count: usize,

    /// Cached data from last snapshot
    pub flows: Vec<FlowSnapshot>,
    /// Aggregated per-process bandwidth snapshots
    pub process_snapshots: Vec<ProcessSnapshot>,
    /// Selected index in process view
    pub process_selected: Option<usize>,
    /// Scroll offset in process view
    pub process_scroll: usize,
    pub totals: TotalStats,
    pub resolver: Resolver,
}

impl AppState {
    pub fn new(
        resolver: Resolver, show_ports: bool, show_bars: bool,
        use_bytes: bool, show_processes: bool, prefs: &Prefs,
        cli_overrides: CliOverrides,
    ) -> Self {
        let theme_name = prefs.theme;
        let theme = if let Some(ref name) = prefs.active_custom_theme
            && let Some(ct) = prefs.custom_themes.get(name) {
            Theme::from_palette_raw(ct.c1, ct.c2, ct.c3, ct.c4, ct.c5, ct.c6)
        } else {
            Theme::from_name(theme_name)
        };
        AppState {
            show_dns: resolver.is_enabled(),
            show_port_names: true,
            show_ports, show_bars,
            show_cumulative: prefs.show_cumulative,
            show_processes, use_bytes,
            sort_column: SortColumn::Avg2s,
            sort_reverse: false,
            line_display: LineDisplay::TwoLine,
            bar_style: prefs.bar_style,
            paused: false,
            scroll_offset: 0,
            selected: None,
            show_help: false,
            screen_filter: None,
            frozen_order: false,
            theme_name,
            theme,
            theme_chooser: ThemeChooser::new(),
            theme_edit: ThemeEditState::new(),
            filter_state: FilterState::new(),
            interface_chooser: InterfaceChooser::new(),
            status_msg: None,
            pinned: prefs.pinned.clone(),
            tooltip: Tooltip::new(),
            show_border: prefs.show_border,
            show_header: prefs.show_header,
            flow_area_y: 2,
            header_bar_y: 0,
            hover: HoverState::default(),
            alert_state: AlertState::default(),
            alert_threshold: prefs.alert_threshold,
            refresh_rate: prefs.refresh_rate,
            interface_name: String::new(),
            config_interface: prefs.interface.clone(),
            custom_themes: prefs.custom_themes.clone(),
            active_custom_theme: prefs.active_custom_theme.clone(),
            orig_prefs: prefs.clone(),
            cli_overrides,
            view_tab: ViewTab::Flows,
            total_flow_count: 0,
            flows: Vec::new(),
            process_snapshots: Vec::new(),
            process_selected: None,
            process_scroll: 0,
            totals: TotalStats {
                sent_2s: 0.0, sent_10s: 0.0, sent_40s: 0.0,
                recv_2s: 0.0, recv_10s: 0.0, recv_40s: 0.0,
                cumulative_sent: 0, cumulative_recv: 0,
                peak_sent: 0.0, peak_recv: 0.0,
            },
            resolver,
        }
    }

    pub fn set_theme(&mut self, name: ThemeName) {
        self.theme_name = name;
        self.theme = Theme::from_name(name);
        self.active_custom_theme = None;
    }

    pub fn apply_custom_palette(&mut self, colors: [u8; 6]) {
        self.theme = Theme::from_palette_raw(colors[0], colors[1], colors[2], colors[3], colors[4], colors[5]);
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = Some(StatusMsg::new(msg.into()));
    }

    pub fn save_prefs(&self) {
        let co = &self.cli_overrides;
        let op = &self.orig_prefs;
        let p = Prefs {
            theme: self.theme_name,
            dns_resolution: if co.dns { op.dns_resolution } else { self.show_dns },
            port_resolution: self.show_port_names,
            show_ports: if co.show_ports { op.show_ports } else { self.show_ports },
            show_bars: if co.show_bars { op.show_bars } else { self.show_bars },
            use_bytes: if co.use_bytes { op.use_bytes } else { self.use_bytes },
            show_processes: if co.show_processes { op.show_processes } else { self.show_processes },
            show_cumulative: self.show_cumulative,
            bar_style: self.bar_style,
            pinned: self.pinned.clone(),
            show_border: self.show_border,
            show_header: self.show_header,
            refresh_rate: self.refresh_rate,
            alert_threshold: self.alert_threshold,
            interface: if co.interface { op.interface.clone() } else { self.config_interface.clone() },
            custom_themes: self.custom_themes.clone(),
            active_custom_theme: self.active_custom_theme.clone(),
        };
        prefs::save_prefs(&p);
    }

    /// Cycle refresh rate: 1 → 2 → 5 → 10 → 1
    pub fn cycle_refresh_rate(&mut self) {
        self.refresh_rate = match self.refresh_rate {
            1 => 2, 2 => 5, 5 => 10, _ => 1,
        };
        self.set_status(format!("Refresh rate: {}s", self.refresh_rate));
        self.save_prefs();
    }

    /// Generate tooltip lines for a hovered header bar segment.
    pub fn header_segment_tooltip(&self, segment: &str) -> Vec<(String, String)> {
        let seg = segment.to_lowercase();
        if seg.contains("iftoprs") {
            vec![
                ("▶ App".into(), "IFTOPRS".into()),
                ("  Version".into(), format!("v{}", env!("CARGO_PKG_VERSION"))),
                ("  Desc".into(), "Real-time bandwidth monitor".into()),
                ("  Author".into(), "MenkeTechnologies".into()),
                ("  License".into(), "MIT".into()),
                ("  Repo".into(), "github.com/MenkeTechnologies/iftoprs".into()),
            ]
        } else if seg.starts_with("iface:") {
            let iface = if self.interface_name.is_empty() { "auto-detected" } else { &self.interface_name };
            vec![
                ("▶ Interface".into(), iface.to_string()),
                ("  Mode".into(), if self.interface_name.is_empty() { "Auto (default gateway)" } else { "Manual (-i flag)" }.into()),
                ("  DNS".into(), if self.show_dns { "Enabled (n to toggle)" } else { "Disabled" }.into()),
                ("  Ports".into(), if self.show_ports { "Shown (p to toggle)" } else { "Hidden" }.into()),
                ("  Promisc".into(), "Set via -p flag".into()),
            ]
        } else if seg.starts_with("flows:") {
            let filtered = self.flows.len();
            let pinned = self.pinned.len();
            let total_rate = self.totals.sent_2s + self.totals.recv_2s;
            vec![
                ("▶ Flows".into(), format!("{} total", self.total_flow_count)),
                ("  Visible".into(), format!("{} (after filter)", filtered)),
                ("  Pinned".into(), format!("{} (F to pin)", pinned)),
                ("  Total TX".into(), crate::util::format::readable_size(self.totals.sent_2s, self.use_bytes)),
                ("  Total RX".into(), crate::util::format::readable_size(self.totals.recv_2s, self.use_bytes)),
                ("  Combined".into(), crate::util::format::readable_size(total_rate, self.use_bytes)),
                ("  Peak TX".into(), crate::util::format::readable_size(self.totals.peak_sent, self.use_bytes)),
                ("  Peak RX".into(), crate::util::format::readable_size(self.totals.peak_recv, self.use_bytes)),
            ]
        } else if seg.starts_with("clock:") {
            let now = chrono::Local::now();
            vec![
                ("▶ Clock".into(), now.format("%H:%M:%S").to_string()),
                ("  Date".into(), now.format("%Y-%m-%d").to_string()),
                ("  Timezone".into(), now.format("%Z").to_string()),
                ("  Uptime".into(), format!("{}s", std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs())),
            ]
        } else if seg.starts_with("sort:") {
            let name = match self.sort_column {
                SortColumn::Avg2s => "2-second average",
                SortColumn::Avg10s => "10-second average",
                SortColumn::Avg40s => "40-second average",
                SortColumn::SrcName => "Source hostname",
                SortColumn::DstName => "Destination hostname",
            };
            vec![
                ("▶ Sort".into(), name.into()),
                ("  Direction".into(), if self.sort_reverse { "Reversed" } else { "Normal (highest first)" }.into()),
                ("  Frozen".into(), if self.frozen_order { "Yes (o to unfreeze)" } else { "No (live re-sort)" }.into()),
                ("  Keys".into(), "1/2/3 = rate, </> = host".into()),
                ("  Reverse".into(), "r to toggle".into()),
            ]
        } else if seg.starts_with("rate:") {
            vec![
                ("▶ Refresh Rate".into(), format!("{}s", self.refresh_rate)),
                ("  Cycle".into(), "f to change (1→2→5→10)".into()),
                ("  Rendering".into(), "~30 fps (33ms)".into()),
                ("  Data".into(), format!("Every {}s", self.refresh_rate)),
            ]
        } else if seg.starts_with("theme:") {
            vec![
                ("▶ Theme".into(), self.theme_name.display_name().into()),
                ("  Available".into(), format!("{} themes", crate::config::theme::ThemeName::ALL.len())),
                ("  Chooser".into(), "c to open".into()),
                ("  CLI".into(), "--list-colors to preview".into()),
            ]
        } else if seg.starts_with("filter:") {
            let filter_text = self.screen_filter.as_deref().unwrap_or("(none)");
            vec![
                ("▶ Filter".into(), filter_text.into()),
                ("  Matched".into(), format!("{} of {} flows", self.flows.len(), self.total_flow_count)),
                ("  Open".into(), "/ to search".into()),
                ("  Clear".into(), "0 to reset".into()),
            ]
        } else if seg.contains("paused") {
            vec![
                ("▶ Paused".into(), "Data refresh is frozen".into()),
                ("  Resume".into(), "P to toggle".into()),
                ("  Display".into(), "Showing last captured state".into()),
            ]
        } else if seg.starts_with("h=help") {
            vec![
                ("▶ Help".into(), "Press h or ? for keybinds".into()),
                ("  Export".into(), "e to export flows".into()),
                ("  Copy".into(), "y to copy selected flow".into()),
                ("  Quit".into(), "q to exit".into()),
            ]
        } else {
            vec![("▶ Header".into(), segment.to_string())]
        }
    }

    /// Check flows against alert threshold and trigger flash if new alerts found.
    pub fn check_alerts(&mut self) {
        if self.alert_threshold <= 0.0 { return; }
        let thresh = self.alert_threshold;
        let mut current = HashSet::new();
        let mut new_alerts = Vec::new();
        for f in &self.flows {
            let rate = f.sent_2s + f.recv_2s;
            if rate >= thresh {
                let key = format!("{}:{}<=>{}:{}", f.key.src, f.key.src_port, f.key.dst, f.key.dst_port);
                if !self.alert_state.alert_flows.contains(&key) {
                    let src = self.resolver.resolve(f.key.src);
                    new_alerts.push(format!("{} {}/s", src,
                        crate::util::format::readable_size(rate, self.use_bytes)));
                }
                current.insert(key);
            }
        }
        if !new_alerts.is_empty() {
            self.alert_state.flash = Some(Instant::now());
            self.set_status(format!("⚠ ALERT: {}", new_alerts.join(", ")));
            print!("\x07"); // terminal bell
        }
        self.alert_state.alert_flows = current;
    }

    /// Toggle pin for the currently selected flow.
    pub fn toggle_pin(&mut self) {
        let idx = match self.selected {
            Some(i) if i < self.flows.len() => i,
            _ => { self.set_status("Select a flow first (j/k)"); return; }
        };
        let f = &self.flows[idx];
        let pin = PinnedFlow {
            src: f.key.src.to_string(),
            dst: f.key.dst.to_string(),
        };
        if let Some(pos) = self.pinned.iter().position(|p| *p == pin) {
            let label = format!("{} <=> {}", pin.src, pin.dst);
            self.pinned.remove(pos);
            self.set_status(format!("Unpinned {}", label));
        } else {
            let label = format!("{} <=> {}", pin.src, pin.dst);
            self.pinned.push(pin);
            self.set_status(format!("Pinned ★ {}", label));
        }
        self.save_prefs();
    }

    /// Show right-click tooltip for a flow.
    pub fn show_tooltip(&mut self, idx: usize, x: u16, y: u16) {
        if idx >= self.flows.len() { return; }
        let f = &self.flows[idx];
        let src = self.format_host(f.key.src, f.key.src_port, &f.key.protocol);
        let dst = self.format_host(f.key.dst, f.key.dst_port, &f.key.protocol);
        let mut lines = Vec::new();
        lines.push(("Source".into(), src));
        lines.push(("Destination".into(), dst));
        lines.push(("Protocol".into(), format!("{}", f.key.protocol)));
        if let (Some(pid), Some(name)) = (f.pid, &f.process_name) {
            lines.push(("Process".into(), format!("[{}:{}]", pid, name)));
        }
        lines.push(("".into(), "".into()));
        lines.push(("TX 2s".into(), crate::util::format::readable_size(f.sent_2s, self.use_bytes)));
        lines.push(("TX 10s".into(), crate::util::format::readable_size(f.sent_10s, self.use_bytes)));
        lines.push(("TX 40s".into(), crate::util::format::readable_size(f.sent_40s, self.use_bytes)));
        lines.push(("TX total".into(), crate::util::format::readable_total(f.total_sent, self.use_bytes)));
        lines.push(("".into(), "".into()));
        lines.push(("RX 2s".into(), crate::util::format::readable_size(f.recv_2s, self.use_bytes)));
        lines.push(("RX 10s".into(), crate::util::format::readable_size(f.recv_10s, self.use_bytes)));
        lines.push(("RX 40s".into(), crate::util::format::readable_size(f.recv_40s, self.use_bytes)));
        lines.push(("RX total".into(), crate::util::format::readable_total(f.total_recv, self.use_bytes)));
        lines.push(("".into(), "".into()));
        lines.push(("Combined".into(), crate::util::format::readable_total(f.total_sent + f.total_recv, self.use_bytes)));
        if self.is_pinned(&f.key) {
            lines.push(("Pinned".into(), "★".into()));
        }
        self.tooltip = Tooltip { active: true, x, y, lines };
    }

    /// Check if a flow is pinned.
    pub fn is_pinned(&self, key: &FlowKey) -> bool {
        let pin = PinnedFlow { src: key.src.to_string(), dst: key.dst.to_string() };
        self.pinned.contains(&pin)
    }

    /// Copy selected flow info to clipboard.
    pub fn copy_selected(&mut self) {
        let idx = match self.selected {
            Some(i) if i < self.flows.len() => i,
            _ => { self.set_status("Select a flow first (j/k)"); return; }
        };
        let f = &self.flows[idx];
        let src = self.format_host(f.key.src, f.key.src_port, &f.key.protocol);
        let dst = self.format_host(f.key.dst, f.key.dst_port, &f.key.protocol);
        let text = format!("{} <=> {} [{}]", src, dst, f.key.protocol);

        let result = if cfg!(target_os = "macos") {
            std::process::Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(ref mut stdin) = child.stdin {
                        stdin.write_all(text.as_bytes())?;
                    }
                    child.wait()
                })
        } else {
            std::process::Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    if let Some(ref mut stdin) = child.stdin {
                        stdin.write_all(text.as_bytes())?;
                    }
                    child.wait()
                })
        };

        match result {
            Ok(_) => self.set_status(format!("Copied: {}", text)),
            Err(e) => self.set_status(format!("Copy failed: {}", e)),
        }
    }

    /// Navigate selection down.
    pub fn select_next(&mut self) {
        let max = self.flows.len().saturating_sub(1);
        self.selected = Some(match self.selected {
            Some(i) => (i + 1).min(max),
            None => 0,
        });
        // Auto-scroll to keep selection visible
        if let Some(sel) = self.selected
            && sel >= self.scroll_offset + 20 { // rough visible count
                self.scroll_offset = sel.saturating_sub(19);
            }
    }

    /// Navigate selection up.
    pub fn select_prev(&mut self) {
        self.selected = Some(match self.selected {
            Some(i) => i.saturating_sub(1),
            None => 0,
        });
        if let Some(sel) = self.selected
            && sel < self.scroll_offset {
                self.scroll_offset = sel;
            }
    }

    /// Half-page down.
    pub fn page_down(&mut self) {
        let half = 10;
        let max = self.flows.len().saturating_sub(1);
        self.selected = Some(match self.selected {
            Some(i) => (i + half).min(max),
            None => half.min(max),
        });
        if let Some(sel) = self.selected
            && sel >= self.scroll_offset + 20 {
                self.scroll_offset = sel.saturating_sub(19);
            }
    }

    /// Half-page up.
    pub fn page_up(&mut self) {
        let half = 10;
        self.selected = Some(match self.selected {
            Some(i) => i.saturating_sub(half),
            None => 0,
        });
        if let Some(sel) = self.selected
            && sel < self.scroll_offset {
                self.scroll_offset = sel;
            }
    }

    /// Jump to first flow.
    pub fn jump_top(&mut self) {
        self.selected = Some(0);
        self.scroll_offset = 0;
    }

    /// Jump to last flow.
    pub fn jump_bottom(&mut self) {
        let last = self.flows.len().saturating_sub(1);
        self.selected = Some(last);
        self.scroll_offset = last.saturating_sub(19);
    }

    // ── Process view navigation ──

    pub fn process_select_next(&mut self) {
        let max = self.process_snapshots.len().saturating_sub(1);
        self.process_selected = Some(match self.process_selected {
            Some(i) => (i + 1).min(max),
            None => 0,
        });
        if let Some(sel) = self.process_selected
            && sel >= self.process_scroll + 20 {
                self.process_scroll = sel.saturating_sub(19);
            }
    }

    pub fn process_select_prev(&mut self) {
        self.process_selected = Some(match self.process_selected {
            Some(i) => i.saturating_sub(1),
            None => 0,
        });
        if let Some(sel) = self.process_selected
            && sel < self.process_scroll {
                self.process_scroll = sel;
            }
    }

    pub fn process_page_down(&mut self) {
        let half = 10;
        let max = self.process_snapshots.len().saturating_sub(1);
        self.process_selected = Some(match self.process_selected {
            Some(i) => (i + half).min(max),
            None => half.min(max),
        });
        if let Some(sel) = self.process_selected
            && sel >= self.process_scroll + 20 {
                self.process_scroll = sel.saturating_sub(19);
            }
    }

    pub fn process_page_up(&mut self) {
        let half = 10;
        self.process_selected = Some(match self.process_selected {
            Some(i) => i.saturating_sub(half),
            None => 0,
        });
        if let Some(sel) = self.process_selected
            && sel < self.process_scroll {
                self.process_scroll = sel;
            }
    }

    pub fn export(&mut self) {
        let path = dirs::home_dir()
            .map(|h| h.join(".iftoprs.export.txt"))
            .unwrap_or_else(|| std::path::PathBuf::from("iftoprs.export.txt"));

        let mut lines = Vec::new();
        lines.push(format!("IFTOPRS EXPORT — {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        lines.push(String::new());
        lines.push(format!("{:<40} {:<6} {:>12} {:>12} {:>12} {:>12}",
            "SOURCE <=> DESTINATION", "PROTO", "TOTAL", "2s", "10s", "40s"));
        lines.push("─".repeat(100));

        for f in &self.flows {
            let src = self.format_host(f.key.src, f.key.src_port, &f.key.protocol);
            let dst = self.format_host(f.key.dst, f.key.dst_port, &f.key.protocol);
            let label = format!("{} <=> {}", src, dst);
            let total = crate::util::format::readable_total(f.total_sent + f.total_recv, self.use_bytes);
            let r2 = crate::util::format::readable_size(f.sent_2s + f.recv_2s, self.use_bytes);
            let r10 = crate::util::format::readable_size(f.sent_10s + f.recv_10s, self.use_bytes);
            let r40 = crate::util::format::readable_size(f.sent_40s + f.recv_40s, self.use_bytes);
            lines.push(format!("{:<40} {:<6} {:>12} {:>12} {:>12} {:>12}",
                if label.len() > 40 { &label[..40] } else { &label },
                f.key.protocol, total, r2, r10, r40));
        }

        lines.push("─".repeat(100));
        let t = &self.totals;
        lines.push(format!("TX  cum: {}  peak: {}  rates: {} / {} / {}",
            crate::util::format::readable_total(t.cumulative_sent, self.use_bytes),
            crate::util::format::readable_size(t.peak_sent, self.use_bytes),
            crate::util::format::readable_size(t.sent_2s, self.use_bytes),
            crate::util::format::readable_size(t.sent_10s, self.use_bytes),
            crate::util::format::readable_size(t.sent_40s, self.use_bytes)));
        lines.push(format!("RX  cum: {}  peak: {}  rates: {} / {} / {}",
            crate::util::format::readable_total(t.cumulative_recv, self.use_bytes),
            crate::util::format::readable_size(t.peak_recv, self.use_bytes),
            crate::util::format::readable_size(t.recv_2s, self.use_bytes),
            crate::util::format::readable_size(t.recv_10s, self.use_bytes),
            crate::util::format::readable_size(t.recv_40s, self.use_bytes)));

        match std::fs::write(&path, lines.join("\n")) {
            Ok(_) => self.set_status(format!("Exported to {}", path.display())),
            Err(e) => self.set_status(format!("Export failed: {}", e)),
        }
    }

    pub fn update_snapshot(&mut self, mut flows: Vec<FlowSnapshot>, totals: TotalStats) {
        if self.paused { return; }
        if let Some(ref msg) = self.status_msg
            && msg.expired() { self.status_msg = None; }

        self.total_flow_count = flows.len();

        if let Some(ref filter) = self.screen_filter {
            let re = regex::Regex::new(&format!("(?i){}", regex::escape(filter)));
            if let Ok(re) = re {
                flows.retain(|f| {
                    let src = self.resolver.resolve(f.key.src);
                    let dst = self.resolver.resolve(f.key.dst);
                    re.is_match(&src) || re.is_match(&dst)
                });
            }
        }

        if !self.frozen_order { self.sort_flows(&mut flows); }

        // Float pinned flows to top
        if !self.pinned.is_empty() {
            flows.sort_by_key(|f| if self.is_pinned(&f.key) { 0 } else { 1 });
        }

        self.flows = flows;
        self.totals = totals;

        // Aggregate per-process bandwidth
        self.aggregate_processes();

        // Check bandwidth alerts
        self.check_alerts();

        // Clamp selection
        if let Some(sel) = self.selected
            && sel >= self.flows.len() && !self.flows.is_empty() {
                self.selected = Some(self.flows.len() - 1);
            }
    }

    fn sort_flows(&self, flows: &mut [FlowSnapshot]) {
        let rev = self.sort_reverse;
        match self.sort_column {
            SortColumn::Avg2s => flows.sort_by(|a, b| {
                let ord = (b.sent_2s + b.recv_2s).partial_cmp(&(a.sent_2s + a.recv_2s)).unwrap_or(std::cmp::Ordering::Equal);
                if rev { ord.reverse() } else { ord }
            }),
            SortColumn::Avg10s => flows.sort_by(|a, b| {
                let ord = (b.sent_10s + b.recv_10s).partial_cmp(&(a.sent_10s + a.recv_10s)).unwrap_or(std::cmp::Ordering::Equal);
                if rev { ord.reverse() } else { ord }
            }),
            SortColumn::Avg40s => flows.sort_by(|a, b| {
                let ord = (b.sent_40s + b.recv_40s).partial_cmp(&(a.sent_40s + a.recv_40s)).unwrap_or(std::cmp::Ordering::Equal);
                if rev { ord.reverse() } else { ord }
            }),
            SortColumn::SrcName => flows.sort_by(|a, b| {
                let ord = self.resolver.resolve(a.key.src).cmp(&self.resolver.resolve(b.key.src));
                if rev { ord.reverse() } else { ord }
            }),
            SortColumn::DstName => flows.sort_by(|a, b| {
                let ord = self.resolver.resolve(a.key.dst).cmp(&self.resolver.resolve(b.key.dst));
                if rev { ord.reverse() } else { ord }
            }),
        }
    }

    fn aggregate_processes(&mut self) {
        use std::collections::HashMap;
        let mut map: HashMap<String, ProcessSnapshot> = HashMap::new();
        for f in &self.flows {
            let name = f.process_name.clone().unwrap_or_else(|| "(unknown)".to_string());
            let entry = map.entry(name.clone()).or_insert_with(|| ProcessSnapshot {
                name,
                pid: f.pid,
                flow_count: 0,
                sent_2s: 0.0, sent_10s: 0.0, sent_40s: 0.0,
                recv_2s: 0.0, recv_10s: 0.0, recv_40s: 0.0,
                total_sent: 0, total_recv: 0,
            });
            entry.flow_count += 1;
            entry.sent_2s += f.sent_2s;
            entry.sent_10s += f.sent_10s;
            entry.sent_40s += f.sent_40s;
            entry.recv_2s += f.recv_2s;
            entry.recv_10s += f.recv_10s;
            entry.recv_40s += f.recv_40s;
            entry.total_sent += f.total_sent;
            entry.total_recv += f.total_recv;
            // Keep the most recent PID
            if f.pid.is_some() {
                entry.pid = f.pid;
            }
        }
        let mut procs: Vec<ProcessSnapshot> = map.into_values().collect();
        procs.sort_by(|a, b| {
            let ra = a.sent_2s + a.recv_2s;
            let rb = b.sent_2s + b.recv_2s;
            rb.partial_cmp(&ra).unwrap_or(std::cmp::Ordering::Equal)
        });
        self.process_snapshots = procs;

        // Clamp process selection
        if let Some(sel) = self.process_selected
            && sel >= self.process_snapshots.len() && !self.process_snapshots.is_empty() {
                self.process_selected = Some(self.process_snapshots.len() - 1);
            }
    }

    pub fn format_host(&self, addr: std::net::IpAddr, port: u16, protocol: &Protocol) -> String {
        let hostname = self.resolver.resolve(addr);
        if self.show_ports && port > 0 {
            let port_str = if self.show_port_names {
                let is_tcp = matches!(protocol, Protocol::Tcp);
                crate::util::resolver::port_to_service(port, is_tcp)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| port.to_string())
            } else {
                port.to_string()
            };
            format!("{}:{}", hostname, port_str)
        } else {
            hostname
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::tracker::TotalStats;

    fn dummy_prefs() -> Prefs {
        Prefs::default()
    }

    fn make_app() -> AppState {
        let resolver = Resolver::new(false);
        AppState::new(resolver, true, true, false, true, &dummy_prefs(), CliOverrides::default())
    }

    fn make_flow(src_port: u16) -> FlowSnapshot {
        FlowSnapshot {
            key: FlowKey {
                src: "10.0.0.1".parse().unwrap(),
                dst: "10.0.0.2".parse().unwrap(),
                src_port,
                dst_port: 80,
                protocol: Protocol::Tcp,
            },
            sent_2s: src_port as f64 * 100.0,
            sent_10s: 0.0, sent_40s: 0.0,
            recv_2s: 0.0, recv_10s: 0.0, recv_40s: 0.0,
            total_sent: 1000, total_recv: 500,
            process_name: None, pid: None,
        }
    }

    fn zero_totals() -> TotalStats {
        TotalStats {
            sent_2s: 0.0, sent_10s: 0.0, sent_40s: 0.0,
            recv_2s: 0.0, recv_10s: 0.0, recv_40s: 0.0,
            cumulative_sent: 0, cumulative_recv: 0,
            peak_sent: 0.0, peak_recv: 0.0,
        }
    }

    // ── LineDisplay ──

    #[test]
    fn line_display_cycles() {
        let mut d = LineDisplay::TwoLine;
        d = d.next(); assert_eq!(d, LineDisplay::OneLine);
        d = d.next(); assert_eq!(d, LineDisplay::SentOnly);
        d = d.next(); assert_eq!(d, LineDisplay::RecvOnly);
        d = d.next(); assert_eq!(d, LineDisplay::TwoLine);
    }

    // ── BarStyle ──

    #[test]
    fn bar_style_cycles() {
        let mut b = BarStyle::Gradient;
        b = b.next(); assert_eq!(b, BarStyle::Solid);
        b = b.next(); assert_eq!(b, BarStyle::Thin);
        b = b.next(); assert_eq!(b, BarStyle::Ascii);
        b = b.next(); assert_eq!(b, BarStyle::Gradient);
    }

    #[test]
    fn bar_style_names() {
        assert_eq!(BarStyle::Gradient.name(), "gradient");
        assert_eq!(BarStyle::Solid.name(), "solid");
        assert_eq!(BarStyle::Thin.name(), "thin");
        assert_eq!(BarStyle::Ascii.name(), "ascii");
    }

    #[test]
    fn bar_style_default() {
        assert_eq!(BarStyle::default(), BarStyle::Gradient);
    }

    // ── FilterState ──

    #[test]
    fn filter_state_new_is_inactive() {
        let f = FilterState::new();
        assert!(!f.active);
        assert!(f.buf.is_empty());
        assert_eq!(f.cursor, 0);
    }

    #[test]
    fn filter_state_open_copies_current() {
        let mut f = FilterState::new();
        f.open(&Some("test".to_string()));
        assert!(f.active);
        assert_eq!(f.buf, "test");
        assert_eq!(f.cursor, 4);
        assert_eq!(f.prev, Some("test".to_string()));
    }

    #[test]
    fn filter_state_open_none() {
        let mut f = FilterState::new();
        f.open(&None);
        assert!(f.active);
        assert!(f.buf.is_empty());
    }

    #[test]
    fn filter_state_insert() {
        let mut f = FilterState::new();
        f.insert('a');
        f.insert('b');
        f.insert('c');
        assert_eq!(f.buf, "abc");
        assert_eq!(f.cursor, 3);
    }

    #[test]
    fn filter_state_backspace() {
        let mut f = FilterState::new();
        f.insert('a');
        f.insert('b');
        f.backspace();
        assert_eq!(f.buf, "a");
        assert_eq!(f.cursor, 1);
    }

    #[test]
    fn filter_state_backspace_at_start() {
        let mut f = FilterState::new();
        f.backspace();
        assert!(f.buf.is_empty());
    }

    #[test]
    fn filter_state_home_end() {
        let mut f = FilterState::new();
        f.insert('a'); f.insert('b'); f.insert('c');
        f.home(); assert_eq!(f.cursor, 0);
        f.end(); assert_eq!(f.cursor, 3);
    }

    #[test]
    fn filter_state_left_right() {
        let mut f = FilterState::new();
        f.insert('a'); f.insert('b');
        f.left(); assert_eq!(f.cursor, 1);
        f.left(); assert_eq!(f.cursor, 0);
        f.left(); assert_eq!(f.cursor, 0); // clamp
        f.right(); assert_eq!(f.cursor, 1);
        f.right(); assert_eq!(f.cursor, 2);
        f.right(); assert_eq!(f.cursor, 2); // clamp
    }

    #[test]
    fn filter_state_kill_to_end() {
        let mut f = FilterState::new();
        f.buf = "hello world".to_string();
        f.cursor = 5;
        f.kill_to_end();
        assert_eq!(f.buf, "hello");
    }

    #[test]
    fn filter_state_delete_word() {
        let mut f = FilterState::new();
        f.buf = "hello world".to_string();
        f.cursor = 11;
        f.delete_word();
        assert_eq!(f.buf, "hello ");  // Ctrl+W deletes the word, preserves preceding space
    }

    // ── StatusMsg ──

    #[test]
    fn status_msg_not_immediately_expired() {
        let msg = StatusMsg::new("test".to_string());
        assert!(!msg.expired());
        assert_eq!(msg.text, "test");
    }

    // ── ThemeChooser ──

    #[test]
    fn theme_chooser_open_selects_current() {
        let mut tc = ThemeChooser::new();
        assert!(!tc.active);
        tc.open(ThemeName::BladeRunner);
        assert!(tc.active);
        let expected = ThemeName::ALL.iter().position(|&t| t == ThemeName::BladeRunner).unwrap();
        assert_eq!(tc.selected, expected);
    }

    // ── Tooltip ──

    #[test]
    fn tooltip_new_is_inactive() {
        let t = Tooltip::new();
        assert!(!t.active);
        assert!(t.lines.is_empty());
    }

    // ── PinnedFlow ──

    #[test]
    fn pinned_flow_equality() {
        let a = PinnedFlow { src: "10.0.0.1".into(), dst: "10.0.0.2".into() };
        let b = PinnedFlow { src: "10.0.0.1".into(), dst: "10.0.0.2".into() };
        assert_eq!(a, b);
    }

    #[test]
    fn pinned_flow_inequality() {
        let a = PinnedFlow { src: "10.0.0.1".into(), dst: "10.0.0.2".into() };
        let b = PinnedFlow { src: "10.0.0.1".into(), dst: "10.0.0.3".into() };
        assert_ne!(a, b);
    }

    // ── Navigation ──

    #[test]
    fn select_next_from_none() {
        let mut app = make_app();
        app.flows = vec![make_flow(1), make_flow(2), make_flow(3)];
        app.select_next();
        assert_eq!(app.selected, Some(0));
    }

    #[test]
    fn select_next_increments() {
        let mut app = make_app();
        app.flows = vec![make_flow(1), make_flow(2), make_flow(3)];
        app.selected = Some(0);
        app.select_next();
        assert_eq!(app.selected, Some(1));
    }

    #[test]
    fn select_next_clamps_at_end() {
        let mut app = make_app();
        app.flows = vec![make_flow(1), make_flow(2)];
        app.selected = Some(1);
        app.select_next();
        assert_eq!(app.selected, Some(1));
    }

    #[test]
    fn select_prev_decrements() {
        let mut app = make_app();
        app.flows = vec![make_flow(1), make_flow(2), make_flow(3)];
        app.selected = Some(2);
        app.select_prev();
        assert_eq!(app.selected, Some(1));
    }

    #[test]
    fn select_prev_clamps_at_start() {
        let mut app = make_app();
        app.flows = vec![make_flow(1)];
        app.selected = Some(0);
        app.select_prev();
        assert_eq!(app.selected, Some(0));
    }

    #[test]
    fn jump_top_and_bottom() {
        let mut app = make_app();
        app.flows = (0..50).map(make_flow).collect();
        app.jump_bottom();
        assert_eq!(app.selected, Some(49));
        app.jump_top();
        assert_eq!(app.selected, Some(0));
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn page_down_moves() {
        let mut app = make_app();
        app.flows = (0..50).map(make_flow).collect();
        app.selected = Some(0);
        app.page_down();
        assert_eq!(app.selected, Some(10));
    }

    #[test]
    fn page_up_moves() {
        let mut app = make_app();
        app.flows = (0..50).map(make_flow).collect();
        app.selected = Some(20);
        app.page_up();
        assert_eq!(app.selected, Some(10));
    }

    #[test]
    fn page_up_clamps_at_zero() {
        let mut app = make_app();
        app.flows = (0..50).map(make_flow).collect();
        app.selected = Some(3);
        app.page_up();
        assert_eq!(app.selected, Some(0));
    }

    // ── Pinning ──

    #[test]
    fn is_pinned_false_by_default() {
        let app = make_app();
        let key = FlowKey {
            src: "10.0.0.1".parse().unwrap(), dst: "10.0.0.2".parse().unwrap(),
            src_port: 5000, dst_port: 80, protocol: Protocol::Tcp,
        };
        assert!(!app.is_pinned(&key));
    }

    #[test]
    fn is_pinned_after_adding() {
        let mut app = make_app();
        app.pinned.push(PinnedFlow { src: "10.0.0.1".into(), dst: "10.0.0.2".into() });
        let key = FlowKey {
            src: "10.0.0.1".parse().unwrap(), dst: "10.0.0.2".parse().unwrap(),
            src_port: 5000, dst_port: 80, protocol: Protocol::Tcp,
        };
        assert!(app.is_pinned(&key));
    }

    // ── Theme ──

    #[test]
    fn set_theme_changes() {
        let mut app = make_app();
        app.set_theme(ThemeName::BladeRunner);
        assert_eq!(app.theme_name, ThemeName::BladeRunner);
    }

    // ── Status ──

    #[test]
    fn set_status_creates_message() {
        let mut app = make_app();
        assert!(app.status_msg.is_none());
        app.set_status("hello");
        assert_eq!(app.status_msg.as_ref().unwrap().text, "hello");
    }

    // ── Snapshot ──

    #[test]
    fn update_snapshot_stores_flows() {
        let mut app = make_app();
        app.update_snapshot(vec![make_flow(1), make_flow(2)], zero_totals());
        assert_eq!(app.flows.len(), 2);
    }

    #[test]
    fn update_snapshot_paused_ignores() {
        let mut app = make_app();
        app.paused = true;
        app.update_snapshot(vec![make_flow(1)], zero_totals());
        assert!(app.flows.is_empty());
    }

    #[test]
    fn update_snapshot_sorts_by_rate() {
        let mut app = make_app();
        app.sort_column = SortColumn::Avg2s;
        app.update_snapshot(vec![make_flow(1), make_flow(5), make_flow(3)], zero_totals());
        assert_eq!(app.flows[0].key.src_port, 5);
        assert_eq!(app.flows[1].key.src_port, 3);
        assert_eq!(app.flows[2].key.src_port, 1);
    }

    #[test]
    fn update_snapshot_pinned_float_to_top() {
        let mut app = make_app();
        app.pinned.push(PinnedFlow { src: "10.0.0.1".into(), dst: "10.0.0.2".into() });
        app.update_snapshot(vec![make_flow(5), make_flow(1)], zero_totals());
        // Both match the pin (same src/dst), so sort order preserved
        assert_eq!(app.flows.len(), 2);
    }

    #[test]
    fn update_snapshot_frozen_order() {
        let mut app = make_app();
        app.frozen_order = true;
        app.update_snapshot(vec![make_flow(1), make_flow(5), make_flow(3)], zero_totals());
        assert_eq!(app.flows[0].key.src_port, 1);
        assert_eq!(app.flows[1].key.src_port, 5);
        assert_eq!(app.flows[2].key.src_port, 3);
    }

    #[test]
    fn update_snapshot_clamps_selection() {
        let mut app = make_app();
        app.selected = Some(10);
        app.update_snapshot(vec![make_flow(1), make_flow(2)], zero_totals());
        assert_eq!(app.selected, Some(1));
    }

    // ── Format host ──

    #[test]
    fn format_host_no_port() {
        let mut app = make_app();
        app.show_ports = false;
        assert_eq!(app.format_host("10.0.0.1".parse().unwrap(), 80, &Protocol::Tcp), "10.0.0.1");
    }

    #[test]
    fn format_host_with_port() {
        let mut app = make_app();
        app.show_ports = true;
        app.show_port_names = false;
        assert_eq!(app.format_host("10.0.0.1".parse().unwrap(), 8080, &Protocol::Tcp), "10.0.0.1:8080");
    }

    #[test]
    fn format_host_port_zero_hidden() {
        let app = make_app();
        assert_eq!(app.format_host("10.0.0.1".parse().unwrap(), 0, &Protocol::Tcp), "10.0.0.1");
    }

    // ── Sort reverse ──

    #[test]
    fn sort_reverse_flips_order() {
        let mut app = make_app();
        app.sort_column = SortColumn::Avg2s;
        app.sort_reverse = true;
        app.update_snapshot(vec![make_flow(1), make_flow(5), make_flow(3)], zero_totals());
        assert_eq!(app.flows[0].key.src_port, 1);
        assert_eq!(app.flows[1].key.src_port, 3);
        assert_eq!(app.flows[2].key.src_port, 5);
    }

    // ── Border ──

    #[test]
    fn show_border_default_true() {
        let app = make_app();
        assert!(app.show_border);
    }

    #[test]
    fn show_border_toggles() {
        let mut app = make_app();
        assert!(app.show_border);
        app.show_border = false;
        assert!(!app.show_border);
        app.show_border = true;
        assert!(app.show_border);
    }

    // ── Pause ──

    #[test]
    fn paused_default_false() {
        let app = make_app();
        assert!(!app.paused);
    }

    #[test]
    fn paused_blocks_snapshot() {
        let mut app = make_app();
        app.update_snapshot(vec![make_flow(1)], zero_totals());
        assert_eq!(app.flows.len(), 1);
        app.paused = true;
        app.update_snapshot(vec![make_flow(1), make_flow(2), make_flow(3)], zero_totals());
        assert_eq!(app.flows.len(), 1); // unchanged because paused
    }

    // ── Prefs round-trip ──

    #[test]
    fn prefs_default_has_border() {
        let p = Prefs::default();
        assert!(p.show_border);
        assert!(p.show_processes);
        assert!(p.show_bars);
        assert!(p.show_ports);
    }

    #[test]
    fn prefs_serializes() {
        let p = Prefs::default();
        let s = toml::to_string_pretty(&p).unwrap();
        assert!(s.contains("show_border"));
        assert!(s.contains("show_processes"));
        assert!(s.contains("show_header"));
        assert!(s.contains("refresh_rate"));
        assert!(s.contains("alert_threshold"));
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.show_border, p.show_border);
        assert_eq!(p2.show_processes, p.show_processes);
        assert_eq!(p2.show_header, p.show_header);
        assert_eq!(p2.refresh_rate, p.refresh_rate);
    }

    // ── Header ──

    #[test]
    fn show_header_default_true() {
        let app = make_app();
        assert!(app.show_header);
    }

    #[test]
    fn show_header_toggles() {
        let mut app = make_app();
        assert!(app.show_header);
        app.show_header = false;
        assert!(!app.show_header);
    }

    // ── Refresh rate ──

    #[test]
    fn refresh_rate_default_1() {
        let app = make_app();
        assert_eq!(app.refresh_rate, 1);
    }

    #[test]
    fn refresh_rate_cycles() {
        let mut app = make_app();
        app.cycle_refresh_rate();
        assert_eq!(app.refresh_rate, 2);
        app.cycle_refresh_rate();
        assert_eq!(app.refresh_rate, 5);
        app.cycle_refresh_rate();
        assert_eq!(app.refresh_rate, 10);
        app.cycle_refresh_rate();
        assert_eq!(app.refresh_rate, 1);
    }

    // ── Alert system ──

    #[test]
    fn alert_state_default_not_flashing() {
        let alert = AlertState::default();
        assert!(!alert.is_flashing());
        assert!(alert.alert_flows.is_empty());
    }

    #[test]
    fn alert_threshold_default_disabled() {
        let app = make_app();
        assert_eq!(app.alert_threshold, 0.0);
    }

    #[test]
    fn alert_no_trigger_when_disabled() {
        let mut app = make_app();
        app.alert_threshold = 0.0;
        app.update_snapshot(vec![make_flow(100)], zero_totals());
        assert!(app.alert_state.flash.is_none());
    }

    #[test]
    fn alert_triggers_when_exceeded() {
        let mut app = make_app();
        // make_flow(100) => sent_2s = 100 * 100.0 = 10000.0
        app.alert_threshold = 5000.0;
        app.update_snapshot(vec![make_flow(100)], zero_totals());
        assert!(app.alert_state.flash.is_some());
        assert!(!app.alert_state.alert_flows.is_empty());
    }

    #[test]
    fn alert_no_trigger_when_below_threshold() {
        let mut app = make_app();
        // make_flow(1) => sent_2s = 100.0
        app.alert_threshold = 5000.0;
        app.update_snapshot(vec![make_flow(1)], zero_totals());
        assert!(app.alert_state.flash.is_none());
        assert!(app.alert_state.alert_flows.is_empty());
    }

    #[test]
    fn total_flow_count_tracked() {
        let mut app = make_app();
        app.update_snapshot(vec![make_flow(1), make_flow(2), make_flow(3)], zero_totals());
        assert_eq!(app.total_flow_count, 3);
    }

    #[test]
    fn interface_name_default_empty() {
        let app = make_app();
        assert!(app.interface_name.is_empty());
    }

    // ── Prefs new fields ──

    #[test]
    fn prefs_default_has_new_fields() {
        let p = Prefs::default();
        assert!(p.show_header);
        assert_eq!(p.refresh_rate, 1);
        assert_eq!(p.alert_threshold, 0.0);
        assert!(p.interface.is_none());
    }

    // ── InterfaceChooser ──

    #[test]
    fn interface_chooser_new_is_inactive() {
        let ic = InterfaceChooser::new();
        assert!(!ic.active);
        assert_eq!(ic.selected, 0);
        assert!(ic.interfaces.is_empty());
    }

    #[test]
    fn interface_chooser_default() {
        let ic = InterfaceChooser::default();
        assert!(!ic.active);
    }

    // ── CliOverrides ──

    #[test]
    fn cli_overrides_default_all_false() {
        let co = CliOverrides::default();
        assert!(!co.dns);
        assert!(!co.show_ports);
        assert!(!co.show_bars);
        assert!(!co.use_bytes);
        assert!(!co.show_processes);
        assert!(!co.interface);
    }

    // ── Config interface preservation ──

    #[test]
    fn config_interface_preserved_on_save() {
        let mut app = make_app();
        app.config_interface = Some("en0".to_string());
        // save_prefs is no-op in test, but verify the struct is built correctly
        let p = Prefs {
            theme: app.theme_name,
            dns_resolution: app.show_dns,
            port_resolution: app.show_port_names,
            show_ports: app.show_ports,
            show_bars: app.show_bars,
            use_bytes: app.use_bytes,
            show_processes: app.show_processes,
            show_cumulative: app.show_cumulative,
            bar_style: app.bar_style,
            pinned: app.pinned.clone(),
            show_border: app.show_border,
            show_header: app.show_header,
            refresh_rate: app.refresh_rate,
            alert_threshold: app.alert_threshold,
            interface: app.config_interface.clone(),
            custom_themes: app.custom_themes.clone(),
            active_custom_theme: app.active_custom_theme.clone(),
        };
        assert_eq!(p.interface, Some("en0".to_string()));
        let s = toml::to_string_pretty(&p).unwrap();
        assert!(s.contains("interface = \"en0\""));
    }

    #[test]
    fn config_interface_none_omitted_from_toml() {
        let p = Prefs::default();
        assert!(p.interface.is_none());
        let s = toml::to_string_pretty(&p).unwrap();
        assert!(!s.contains("interface"), "None interface should be omitted from TOML");
    }

    #[test]
    fn config_interface_roundtrip() {
        let mut p = Prefs::default();
        p.interface = Some("eth0".to_string());
        let s = toml::to_string_pretty(&p).unwrap();
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.interface, Some("eth0".to_string()));
    }

    // ── save_prefs no-op in test ──

    #[test]
    fn save_prefs_does_not_write_in_test() {
        let mut app = make_app();
        // This should not panic or write to disk
        app.save_prefs();
        app.cycle_refresh_rate();
        // If we got here, save_prefs is correctly a no-op
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;
    use crate::data::tracker::TotalStats;

    fn dummy_prefs() -> Prefs { Prefs::default() }
    fn make_app() -> AppState {
        let resolver = Resolver::new(false);
        AppState::new(resolver, true, true, false, true, &dummy_prefs(), CliOverrides::default())
    }
    fn make_flow(src_port: u16) -> FlowSnapshot {
        FlowSnapshot {
            key: FlowKey { src: "10.0.0.1".parse().unwrap(), dst: "10.0.0.2".parse().unwrap(),
                src_port, dst_port: 80, protocol: Protocol::Tcp },
            sent_2s: src_port as f64 * 100.0, sent_10s: 0.0, sent_40s: 0.0,
            recv_2s: 0.0, recv_10s: 0.0, recv_40s: 0.0,
            total_sent: 1000, total_recv: 500, process_name: None, pid: None,
        }
    }
    fn zero_totals() -> TotalStats {
        TotalStats { sent_2s: 0.0, sent_10s: 0.0, sent_40s: 0.0,
            recv_2s: 0.0, recv_10s: 0.0, recv_40s: 0.0,
            cumulative_sent: 0, cumulative_recv: 0, peak_sent: 0.0, peak_recv: 0.0 }
    }

    // ── HoverState ──

    #[test] fn hover_state_default_not_ready() {
        let h = HoverState::default();
        assert!(!h.ready()); assert!(h.pos.is_none()); assert!(!h.right_click);
    }
    #[test] fn hover_state_move_to_sets_position() {
        let mut h = HoverState::default(); h.move_to(10, 20);
        assert_eq!(h.pos, Some((10, 20))); assert!(h.since.is_some());
    }
    #[test] fn hover_state_move_same_no_reset() {
        let mut h = HoverState::default(); h.move_to(10, 20);
        let s = h.since.unwrap(); h.move_to(10, 20); assert_eq!(h.since.unwrap(), s);
    }
    #[test] fn hover_state_move_different_resets() {
        let mut h = HoverState::default(); h.move_to(10, 20);
        let s = h.since.unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1));
        h.move_to(11, 20); assert_ne!(h.since.unwrap(), s);
    }
    #[test] fn hover_right_click_immediately_ready() {
        let mut h = HoverState::default(); h.right_click_at(5, 5);
        assert!(h.right_click); assert!(h.ready());
    }
    #[test] fn hover_not_ready_before_delay() {
        let mut h = HoverState::default(); h.move_to(10, 10); assert!(!h.ready());
    }
    #[test] fn hover_right_click_clears_on_move() {
        let mut h = HoverState::default(); h.right_click_at(5, 5);
        h.move_to(6, 6); assert!(!h.right_click);
    }

    // ── AlertState ──

    #[test] fn alert_flashing_recent() {
        let mut a = AlertState::default(); a.flash = Some(Instant::now()); assert!(a.is_flashing());
    }
    #[test] fn alert_not_flashing_expired() {
        let mut a = AlertState::default();
        a.flash = Some(Instant::now() - std::time::Duration::from_secs(3));
        assert!(!a.is_flashing());
    }

    // ── check_alerts ──

    #[test] fn check_alerts_disabled() {
        let mut app = make_app(); app.alert_threshold = 0.0;
        app.flows = vec![make_flow(100)]; app.check_alerts();
        assert!(app.alert_state.flash.is_none());
    }
    #[test] fn check_alerts_fires() {
        let mut app = make_app(); app.alert_threshold = 50.0;
        app.flows = vec![make_flow(100)]; app.check_alerts();
        assert!(app.alert_state.flash.is_some());
    }
    #[test] fn check_alerts_no_double_fire() {
        let mut app = make_app(); app.alert_threshold = 50.0;
        app.flows = vec![make_flow(100)]; app.check_alerts();
        app.alert_state.flash = None; app.check_alerts();
        assert!(app.alert_state.flash.is_none());
    }
    #[test] fn check_alerts_clears_old() {
        let mut app = make_app(); app.alert_threshold = 50.0;
        app.flows = vec![make_flow(100)]; app.check_alerts();
        app.flows.clear(); app.check_alerts();
        assert!(app.alert_state.alert_flows.is_empty());
    }

    // ── toggle_pin ──

    #[test] fn toggle_pin_no_selection() {
        let mut app = make_app(); app.flows = vec![make_flow(1)];
        app.toggle_pin(); assert!(app.pinned.is_empty());
    }
    #[test] fn toggle_pin_adds() {
        let mut app = make_app(); app.flows = vec![make_flow(1)]; app.selected = Some(0);
        app.toggle_pin(); assert_eq!(app.pinned.len(), 1);
        assert!(app.status_msg.unwrap().text.contains("Pinned"));
    }
    #[test] fn toggle_pin_removes() {
        let mut app = make_app(); app.flows = vec![make_flow(1)]; app.selected = Some(0);
        app.toggle_pin(); app.toggle_pin(); assert!(app.pinned.is_empty());
        assert!(app.status_msg.unwrap().text.contains("Unpinned"));
    }
    #[test] fn toggle_pin_out_of_bounds() {
        let mut app = make_app(); app.flows = vec![make_flow(1)]; app.selected = Some(99);
        app.toggle_pin(); assert!(app.pinned.is_empty());
    }

    // ── show_tooltip ──

    #[test] fn show_tooltip_basic() {
        let mut app = make_app(); app.flows = vec![make_flow(1)];
        app.show_tooltip(0, 10, 5);
        assert!(app.tooltip.active); assert_eq!(app.tooltip.x, 10);
    }
    #[test] fn show_tooltip_oob() {
        let mut app = make_app(); app.flows = vec![make_flow(1)];
        app.show_tooltip(99, 0, 0); assert!(!app.tooltip.active);
    }
    #[test] fn show_tooltip_process() {
        let mut app = make_app();
        let mut f = make_flow(1); f.pid = Some(1234); f.process_name = Some("curl".into());
        app.flows = vec![f]; app.show_tooltip(0, 0, 0);
        assert!(app.tooltip.lines.iter().any(|(l, _)| l == "Process"));
    }
    #[test] fn show_tooltip_pinned() {
        let mut app = make_app(); app.flows = vec![make_flow(1)];
        app.pinned.push(PinnedFlow { src: "10.0.0.1".into(), dst: "10.0.0.2".into() });
        app.show_tooltip(0, 0, 0);
        assert!(app.tooltip.lines.iter().any(|(l, _)| l == "Pinned"));
    }
    #[test] fn show_tooltip_bandwidth_lines() {
        let mut app = make_app(); app.flows = vec![make_flow(1)];
        app.show_tooltip(0, 0, 0);
        assert!(app.tooltip.lines.iter().any(|(l, _)| l == "TX 2s"));
        assert!(app.tooltip.lines.iter().any(|(l, _)| l == "RX 2s"));
        assert!(app.tooltip.lines.iter().any(|(l, _)| l == "Combined"));
    }

    // ── header_segment_tooltip ──

    #[test] fn hdr_iftoprs() {
        let app = make_app();
        let l = app.header_segment_tooltip("IFTOPRS");
        assert!(l[0].1.contains("IFTOPRS"));
    }
    #[test] fn hdr_iface() {
        let mut app = make_app(); app.interface_name = "en0".into();
        let l = app.header_segment_tooltip("iface:en0");
        assert!(l.iter().any(|(_, v)| v.contains("en0")));
    }
    #[test] fn hdr_iface_empty() {
        let l = make_app().header_segment_tooltip("iface:");
        assert!(l.iter().any(|(_, v)| v.contains("auto-detected")));
    }
    #[test] fn hdr_flows() {
        let mut app = make_app(); app.total_flow_count = 42;
        let l = app.header_segment_tooltip("flows:42");
        assert!(l.iter().any(|(_, v)| v.contains("42")));
    }
    #[test] fn hdr_clock() {
        let l = make_app().header_segment_tooltip("clock:12:00");
        assert!(l.iter().any(|(l, _)| l.contains("Clock")));
    }
    #[test] fn hdr_sort_all_columns() {
        for (col, expected) in [
            (SortColumn::Avg2s, "2-second"), (SortColumn::Avg10s, "10-second"),
            (SortColumn::Avg40s, "40-second"), (SortColumn::SrcName, "Source"),
            (SortColumn::DstName, "Destination"),
        ] {
            let mut app = make_app(); app.sort_column = col;
            let l = app.header_segment_tooltip("sort:x");
            assert!(l.iter().any(|(_, v)| v.contains(expected)), "{:?}", col);
        }
    }
    #[test] fn hdr_sort_reversed() {
        let mut app = make_app(); app.sort_reverse = true;
        let l = app.header_segment_tooltip("sort:2s");
        assert!(l.iter().any(|(_, v)| v.contains("Reversed")));
    }
    #[test] fn hdr_sort_frozen() {
        let mut app = make_app(); app.frozen_order = true;
        let l = app.header_segment_tooltip("sort:2s");
        assert!(l.iter().any(|(_, v)| v.contains("Yes")));
    }
    #[test] fn hdr_rate() {
        let mut app = make_app(); app.refresh_rate = 5;
        let l = app.header_segment_tooltip("rate:5s");
        assert!(l.iter().any(|(_, v)| v.contains("5s")));
    }
    #[test] fn hdr_theme() {
        let mut app = make_app(); app.set_theme(ThemeName::BladeRunner);
        let l = app.header_segment_tooltip("theme:blade");
        assert!(l.iter().any(|(_, v)| v.contains("Blade Runner")));
    }
    #[test] fn hdr_filter() {
        let mut app = make_app(); app.screen_filter = Some("tcp".into());
        let l = app.header_segment_tooltip("filter:tcp");
        assert!(l.iter().any(|(_, v)| v.contains("tcp")));
    }
    #[test] fn hdr_filter_none() {
        let l = make_app().header_segment_tooltip("filter:");
        assert!(l.iter().any(|(_, v)| v.contains("(none)")));
    }
    #[test] fn hdr_paused() {
        let l = make_app().header_segment_tooltip("PAUSED");
        assert!(l.iter().any(|(_, v)| v.contains("frozen")));
    }
    #[test] fn hdr_help() {
        let l = make_app().header_segment_tooltip("h=help");
        assert!(l.iter().any(|(l, _)| l.contains("Help")));
    }
    #[test] fn hdr_unknown() {
        let l = make_app().header_segment_tooltip("unknown_segment");
        assert_eq!(l.len(), 1); assert!(l[0].1.contains("unknown_segment"));
    }

    // ── update_snapshot filter ──

    #[test] fn snapshot_filter_match() {
        let mut app = make_app(); app.screen_filter = Some("10.0.0.1".into());
        app.update_snapshot(vec![make_flow(1), make_flow(2)], zero_totals());
        assert_eq!(app.flows.len(), 2);
    }
    #[test] fn snapshot_filter_no_match() {
        let mut app = make_app(); app.screen_filter = Some("172.16.0.0".into());
        app.update_snapshot(vec![make_flow(1), make_flow(2)], zero_totals());
        assert_eq!(app.flows.len(), 0);
    }
    #[test] fn snapshot_total_count_before_filter() {
        let mut app = make_app(); app.screen_filter = Some("172.16.0.0".into());
        app.update_snapshot(vec![make_flow(1), make_flow(2), make_flow(3)], zero_totals());
        assert_eq!(app.total_flow_count, 3); assert_eq!(app.flows.len(), 0);
    }

    // ── Sort columns ──

    #[test] fn sort_avg10s() {
        let mut app = make_app(); app.sort_column = SortColumn::Avg10s;
        let mut f1 = make_flow(1); f1.sent_10s = 100.0;
        let mut f2 = make_flow(2); f2.sent_10s = 500.0;
        app.update_snapshot(vec![f1, f2], zero_totals());
        assert_eq!(app.flows[0].key.src_port, 2);
    }
    #[test] fn sort_avg40s() {
        let mut app = make_app(); app.sort_column = SortColumn::Avg40s;
        let mut f1 = make_flow(1); f1.sent_40s = 300.0;
        let mut f2 = make_flow(2); f2.sent_40s = 100.0;
        app.update_snapshot(vec![f1, f2], zero_totals());
        assert_eq!(app.flows[0].key.src_port, 1);
    }
    #[test] fn sort_src_name() {
        let mut app = make_app(); app.sort_column = SortColumn::SrcName;
        let f1 = FlowSnapshot { key: FlowKey { src: "192.168.1.1".parse().unwrap(), dst: "10.0.0.2".parse().unwrap(), src_port: 1, dst_port: 80, protocol: Protocol::Tcp }, sent_2s: 0.0, sent_10s: 0.0, sent_40s: 0.0, recv_2s: 0.0, recv_10s: 0.0, recv_40s: 0.0, total_sent: 0, total_recv: 0, process_name: None, pid: None };
        let f2 = FlowSnapshot { key: FlowKey { src: "10.0.0.1".parse().unwrap(), dst: "10.0.0.2".parse().unwrap(), src_port: 2, dst_port: 80, protocol: Protocol::Tcp }, sent_2s: 0.0, sent_10s: 0.0, sent_40s: 0.0, recv_2s: 0.0, recv_10s: 0.0, recv_40s: 0.0, total_sent: 0, total_recv: 0, process_name: None, pid: None };
        app.update_snapshot(vec![f1, f2], zero_totals());
        assert_eq!(app.flows[0].key.src_port, 2);
    }
    #[test] fn sort_dst_name() {
        let mut app = make_app(); app.sort_column = SortColumn::DstName;
        let f1 = FlowSnapshot { key: FlowKey { src: "10.0.0.1".parse().unwrap(), dst: "192.168.1.1".parse().unwrap(), src_port: 1, dst_port: 80, protocol: Protocol::Tcp }, sent_2s: 0.0, sent_10s: 0.0, sent_40s: 0.0, recv_2s: 0.0, recv_10s: 0.0, recv_40s: 0.0, total_sent: 0, total_recv: 0, process_name: None, pid: None };
        let f2 = FlowSnapshot { key: FlowKey { src: "10.0.0.1".parse().unwrap(), dst: "10.0.0.2".parse().unwrap(), src_port: 2, dst_port: 80, protocol: Protocol::Tcp }, sent_2s: 0.0, sent_10s: 0.0, sent_40s: 0.0, recv_2s: 0.0, recv_10s: 0.0, recv_40s: 0.0, total_sent: 0, total_recv: 0, process_name: None, pid: None };
        app.update_snapshot(vec![f1, f2], zero_totals());
        assert_eq!(app.flows[0].key.src_port, 2);
    }

    // ── format_host port names ──

    #[test] fn format_host_service_name() {
        let mut app = make_app(); app.show_ports = true; app.show_port_names = true;
        let r = app.format_host("10.0.0.1".parse().unwrap(), 80, &Protocol::Tcp);
        assert!(r.contains("http") || r.contains("80"));
    }
    #[test] fn format_host_unknown_port() {
        let mut app = make_app(); app.show_ports = true; app.show_port_names = true;
        let r = app.format_host("10.0.0.1".parse().unwrap(), 65432, &Protocol::Tcp);
        assert!(r.contains("65432"));
    }

    // ── Defaults ──

    #[test] fn tooltip_default() { let t = Tooltip::default(); assert!(!t.active); }
    #[test] fn filter_state_default() { let f = FilterState::default(); assert!(!f.active); }

    // ── FilterState unicode ──

    #[test] fn filter_unicode_insert() {
        let mut f = FilterState::new(); f.insert('ä'); f.insert('ö');
        assert_eq!(f.buf, "äö"); assert_eq!(f.cursor, f.buf.len());
    }
    #[test] fn filter_unicode_backspace() {
        let mut f = FilterState::new(); f.insert('ä'); f.insert('ö');
        f.backspace(); assert_eq!(f.buf, "ä");
    }
    #[test] fn filter_unicode_left_right() {
        let mut f = FilterState::new(); f.insert('ä'); f.insert('b');
        f.left(); assert_eq!(f.cursor, 2); f.left(); assert_eq!(f.cursor, 0);
        f.right(); assert_eq!(f.cursor, 2);
    }
    #[test] fn filter_mid_insert() {
        let mut f = FilterState::new(); f.insert('a'); f.insert('c');
        f.left(); f.insert('b'); assert_eq!(f.buf, "abc");
    }

    // ── Navigation edge cases ──

    #[test] fn select_next_empty() { let mut app = make_app(); app.select_next(); assert_eq!(app.selected, Some(0)); }
    #[test] fn select_prev_empty() { let mut app = make_app(); app.select_prev(); assert_eq!(app.selected, Some(0)); }
    #[test] fn jump_bottom_empty() { let mut app = make_app(); app.jump_bottom(); assert_eq!(app.selected, Some(0)); }
    #[test] fn scroll_adjusts_next() {
        let mut app = make_app(); app.flows = (0..50).map(make_flow).collect();
        app.selected = Some(19); app.select_next();
        assert_eq!(app.selected, Some(20)); assert!(app.scroll_offset > 0);
    }
    #[test] fn scroll_adjusts_prev() {
        let mut app = make_app(); app.flows = (0..50).map(make_flow).collect();
        app.scroll_offset = 10; app.selected = Some(10);
        app.select_prev(); assert_eq!(app.selected, Some(9)); assert!(app.scroll_offset <= 9);
    }

    // ── Misc ──

    #[test] fn snapshot_stores_totals() {
        let mut app = make_app();
        let t = TotalStats { sent_2s: 100.0, sent_10s: 200.0, sent_40s: 300.0, recv_2s: 50.0, recv_10s: 100.0, recv_40s: 150.0, cumulative_sent: 5000, cumulative_recv: 3000, peak_sent: 500.0, peak_recv: 250.0 };
        app.update_snapshot(vec![make_flow(1)], t);
        assert_eq!(app.totals.cumulative_sent, 5000);
    }
    #[test] fn app_defaults() {
        let app = make_app();
        assert_eq!(app.sort_column, SortColumn::Avg2s);
        assert!(!app.sort_reverse); assert!(!app.paused);
        assert!(app.selected.is_none()); assert!(app.flows.is_empty());
    }
    #[test] fn status_clears_on_snapshot() {
        let mut app = make_app();
        app.status_msg = Some(StatusMsg { text: "old".into(), since: Instant::now() - std::time::Duration::from_secs(5) });
        app.update_snapshot(vec![], zero_totals());
        assert!(app.status_msg.is_none());
    }
    #[test] fn cli_override_preserves() {
        let mut p = Prefs::default(); p.dns_resolution = true;
        let co = CliOverrides { dns: true, ..Default::default() };
        let app = AppState::new(Resolver::new(false), true, true, false, true, &p, co);
        assert!(app.cli_overrides.dns); assert!(app.orig_prefs.dns_resolution);
    }
    #[test] fn pinned_hash() {
        use std::collections::HashSet;
        let mut s = HashSet::new();
        s.insert(PinnedFlow { src: "10.0.0.1".into(), dst: "10.0.0.2".into() });
        assert!(s.contains(&PinnedFlow { src: "10.0.0.1".into(), dst: "10.0.0.2".into() }));
    }
    #[test] fn cycle_rate_status() {
        let mut app = make_app(); app.cycle_refresh_rate();
        assert!(app.status_msg.unwrap().text.contains("Refresh rate"));
    }
    #[test] fn export_works() {
        let mut app = make_app(); app.flows = vec![make_flow(1)]; app.export();
        assert!(app.status_msg.is_some());
    }
    #[test] fn copy_no_selection() {
        let mut app = make_app(); app.copy_selected();
        assert!(app.status_msg.unwrap().text.contains("Select a flow"));
    }
    #[test] fn copy_oob() {
        let mut app = make_app(); app.flows = vec![make_flow(1)]; app.selected = Some(99);
        app.copy_selected(); assert!(app.status_msg.unwrap().text.contains("Select a flow"));
    }
    #[test] fn show_cumulative_default() { assert!(!make_app().show_cumulative); }
    #[test] fn sort_column_variants() {
        assert_eq!(SortColumn::Avg2s, SortColumn::Avg2s);
        assert_ne!(SortColumn::Avg2s, SortColumn::Avg10s);
    }

    // ── ViewTab ──

    #[test]
    fn view_tab_default_is_flows() {
        let app = make_app();
        assert_eq!(app.view_tab, ViewTab::Flows);
    }

    // ── Process aggregation ──

    #[test]
    fn process_aggregation_empty() {
        let mut app = make_app();
        app.update_snapshot(vec![], zero_totals());
        assert!(app.process_snapshots.is_empty());
    }

    #[test]
    fn process_aggregation_groups_by_name() {
        let mut app = make_app();
        let mut f1 = make_flow(1);
        f1.process_name = Some("curl".into());
        f1.pid = Some(100);
        let mut f2 = make_flow(2);
        f2.process_name = Some("curl".into());
        f2.pid = Some(100);
        let mut f3 = make_flow(3);
        f3.process_name = Some("firefox".into());
        f3.pid = Some(200);
        app.update_snapshot(vec![f1, f2, f3], zero_totals());
        assert_eq!(app.process_snapshots.len(), 2);
    }

    #[test]
    fn process_aggregation_sums_rates() {
        let mut app = make_app();
        let mut f1 = make_flow(1);
        f1.process_name = Some("curl".into());
        f1.sent_2s = 100.0;
        f1.recv_2s = 50.0;
        let mut f2 = make_flow(2);
        f2.process_name = Some("curl".into());
        f2.sent_2s = 200.0;
        f2.recv_2s = 75.0;
        app.update_snapshot(vec![f1, f2], zero_totals());
        let proc = &app.process_snapshots[0];
        assert_eq!(proc.name, "curl");
        assert_eq!(proc.flow_count, 2);
        assert_eq!(proc.sent_2s, 300.0);
        assert_eq!(proc.recv_2s, 125.0);
    }

    #[test]
    fn process_aggregation_unknown_process() {
        let mut app = make_app();
        let f = make_flow(1); // no process_name
        app.update_snapshot(vec![f], zero_totals());
        assert_eq!(app.process_snapshots.len(), 1);
        assert_eq!(app.process_snapshots[0].name, "(unknown)");
    }

    #[test]
    fn process_aggregation_sorted_by_rate() {
        let mut app = make_app();
        let mut f1 = make_flow(1);
        f1.process_name = Some("slow".into());
        f1.sent_2s = 10.0;
        let mut f2 = make_flow(2);
        f2.process_name = Some("fast".into());
        f2.sent_2s = 1000.0;
        app.update_snapshot(vec![f1, f2], zero_totals());
        assert_eq!(app.process_snapshots[0].name, "fast");
        assert_eq!(app.process_snapshots[1].name, "slow");
    }

    // ── Process navigation ──

    #[test]
    fn process_select_next_from_none() {
        let mut app = make_app();
        let mut f = make_flow(1);
        f.process_name = Some("test".into());
        app.update_snapshot(vec![f], zero_totals());
        app.process_select_next();
        assert_eq!(app.process_selected, Some(0));
    }

    #[test]
    fn process_select_next_increments() {
        let mut app = make_app();
        let mut f1 = make_flow(1); f1.process_name = Some("a".into());
        let mut f2 = make_flow(2); f2.process_name = Some("b".into());
        app.update_snapshot(vec![f1, f2], zero_totals());
        app.process_selected = Some(0);
        app.process_select_next();
        assert_eq!(app.process_selected, Some(1));
    }

    #[test]
    fn process_select_prev_decrements() {
        let mut app = make_app();
        let mut f1 = make_flow(1); f1.process_name = Some("a".into());
        let mut f2 = make_flow(2); f2.process_name = Some("b".into());
        app.update_snapshot(vec![f1, f2], zero_totals());
        app.process_selected = Some(1);
        app.process_select_prev();
        assert_eq!(app.process_selected, Some(0));
    }

    #[test]
    fn process_select_clamps() {
        let mut app = make_app();
        let mut f = make_flow(1); f.process_name = Some("a".into());
        app.update_snapshot(vec![f], zero_totals());
        app.process_selected = Some(0);
        app.process_select_next(); // only 1 item
        assert_eq!(app.process_selected, Some(0));
    }

    #[test]
    fn process_page_down_up() {
        let mut app = make_app();
        let flows: Vec<_> = (0..30).map(|i| {
            let mut f = make_flow(i);
            f.process_name = Some(format!("proc{}", i));
            f
        }).collect();
        app.update_snapshot(flows, zero_totals());
        app.process_selected = Some(0);
        app.process_page_down();
        assert_eq!(app.process_selected, Some(10));
        app.process_page_up();
        assert_eq!(app.process_selected, Some(0));
    }

    #[test]
    fn process_selection_clamps_on_snapshot() {
        let mut app = make_app();
        let mut f1 = make_flow(1); f1.process_name = Some("a".into());
        let mut f2 = make_flow(2); f2.process_name = Some("b".into());
        app.update_snapshot(vec![f1, f2], zero_totals());
        app.process_selected = Some(10); // out of bounds
        let mut f3 = make_flow(3); f3.process_name = Some("c".into());
        app.update_snapshot(vec![f3], zero_totals());
        assert_eq!(app.process_selected, Some(0));
    }
}
