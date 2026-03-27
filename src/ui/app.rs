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
        let new_end = self.buf[..self.cursor].trim_end_matches(|c: char| !c.is_whitespace()).trim_end().len();
        self.buf.drain(new_end..self.cursor);
        self.cursor = new_end;
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
