use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::config::prefs::{self, Prefs};
use crate::config::theme::{Theme, ThemeName};
use crate::data::flow::{FlowKey, Protocol};
use crate::data::tracker::{FlowSnapshot, TotalStats};
use crate::util::resolver::Resolver;

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

/// A flow identity for pinning (favorites).
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PinnedFlow {
    pub src: String,
    pub dst: String,
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
    pub filter_state: FilterState,
    pub status_msg: Option<StatusMsg>,
    pub pinned: Vec<PinnedFlow>,
    pub tooltip: Tooltip,
    pub show_border: bool,
    /// Y offset where the flow area starts (set by renderer).
    pub flow_area_y: u16,

    /// Cached data from last snapshot
    pub flows: Vec<FlowSnapshot>,
    pub totals: TotalStats,
    pub resolver: Resolver,
}

impl AppState {
    pub fn new(
        resolver: Resolver, show_ports: bool, show_bars: bool,
        use_bytes: bool, show_processes: bool, prefs: &Prefs,
    ) -> Self {
        let theme_name = prefs.theme;
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
            theme: Theme::from_name(theme_name),
            theme_chooser: ThemeChooser::new(),
            filter_state: FilterState::new(),
            status_msg: None,
            pinned: prefs.pinned.clone(),
            tooltip: Tooltip::new(),
            show_border: prefs.show_border,
            flow_area_y: 2,
            flows: Vec::new(),
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
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = Some(StatusMsg::new(msg.into()));
    }

    pub fn save_prefs(&self) {
        let p = Prefs {
            theme: self.theme_name,
            dns_resolution: self.show_dns,
            port_resolution: self.show_port_names,
            show_ports: self.show_ports,
            show_bars: self.show_bars,
            use_bytes: self.use_bytes,
            show_processes: self.show_processes,
            show_cumulative: self.show_cumulative,
            bar_style: self.bar_style,
            pinned: self.pinned.clone(),
            show_border: self.show_border,
        };
        prefs::save_prefs(&p);
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
        AppState::new(resolver, true, true, false, true, &dummy_prefs())
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
        let p2: Prefs = toml::from_str(&s).unwrap();
        assert_eq!(p2.show_border, p.show_border);
        assert_eq!(p2.show_processes, p.show_processes);
    }
}
