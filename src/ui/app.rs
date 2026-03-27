use crate::data::flow::Protocol;
use crate::data::tracker::{FlowSnapshot, TotalStats};
use crate::util::resolver::Resolver;

/// Which bandwidth column to sort by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortColumn {
    Avg2s,
    Avg10s,
    Avg40s,
    SrcName,
    DstName,
}

/// Line display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineDisplay {
    TwoLine,
    OneLine,
    SentOnly,
    RecvOnly,
}

impl LineDisplay {
    pub fn next(self) -> Self {
        match self {
            LineDisplay::TwoLine => LineDisplay::OneLine,
            LineDisplay::OneLine => LineDisplay::SentOnly,
            LineDisplay::SentOnly => LineDisplay::RecvOnly,
            LineDisplay::RecvOnly => LineDisplay::TwoLine,
        }
    }
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
    pub line_display: LineDisplay,
    pub paused: bool,
    pub scroll_offset: usize,
    pub show_help: bool,
    pub screen_filter: Option<String>,
    pub frozen_order: bool,

    /// Cached data from last snapshot
    pub flows: Vec<FlowSnapshot>,
    pub totals: TotalStats,
    pub resolver: Resolver,
}

impl AppState {
    pub fn new(
        resolver: Resolver,
        show_ports: bool,
        show_bars: bool,
        use_bytes: bool,
        show_processes: bool,
    ) -> Self {
        AppState {
            show_dns: resolver.is_enabled(),
            show_port_names: true,
            show_ports,
            show_bars,
            show_cumulative: false,
            show_processes,
            use_bytes,
            sort_column: SortColumn::Avg2s,
            line_display: LineDisplay::TwoLine,
            paused: false,
            scroll_offset: 0,
            show_help: false,
            screen_filter: None,
            frozen_order: false,
            flows: Vec::new(),
            totals: TotalStats {
                sent_2s: 0.0,
                sent_10s: 0.0,
                sent_40s: 0.0,
                recv_2s: 0.0,
                recv_10s: 0.0,
                recv_40s: 0.0,
                cumulative_sent: 0,
                cumulative_recv: 0,
                peak_sent: 0.0,
                peak_recv: 0.0,
            },
            resolver,
        }
    }

    /// Update the snapshot from the tracker.
    pub fn update_snapshot(&mut self, mut flows: Vec<FlowSnapshot>, totals: TotalStats) {
        if self.paused {
            return;
        }

        // Apply screen filter
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

        // Sort unless frozen
        if !self.frozen_order {
            self.sort_flows(&mut flows);
        }

        self.flows = flows;
        self.totals = totals;
    }

    fn sort_flows(&self, flows: &mut Vec<FlowSnapshot>) {
        match self.sort_column {
            SortColumn::Avg2s => {
                flows.sort_by(|a, b| {
                    let a_total = a.sent_2s + a.recv_2s;
                    let b_total = b.sent_2s + b.recv_2s;
                    b_total.partial_cmp(&a_total).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            SortColumn::Avg10s => {
                flows.sort_by(|a, b| {
                    let a_total = a.sent_10s + a.recv_10s;
                    let b_total = b.sent_10s + b.recv_10s;
                    b_total.partial_cmp(&a_total).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            SortColumn::Avg40s => {
                flows.sort_by(|a, b| {
                    let a_total = a.sent_40s + a.recv_40s;
                    let b_total = b.sent_40s + b.recv_40s;
                    b_total.partial_cmp(&a_total).unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            SortColumn::SrcName => {
                flows.sort_by(|a, b| {
                    let a_name = self.resolver.resolve(a.key.src);
                    let b_name = self.resolver.resolve(b.key.src);
                    a_name.cmp(&b_name)
                });
            }
            SortColumn::DstName => {
                flows.sort_by(|a, b| {
                    let a_name = self.resolver.resolve(a.key.dst);
                    let b_name = self.resolver.resolve(b.key.dst);
                    a_name.cmp(&b_name)
                });
            }
        }
    }

    /// Format a host address with optional port.
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
