use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::ui::app::AppState;
use crate::util::format::{readable_size, readable_total};

// Color scheme matching iftopcolor screenshot
const COLOR_BAR_SENT: Color = Color::Blue;
// const COLOR_BAR_RECV: Color = Color::Green; // reserved for two-line mode
const COLOR_HOST1: Color = Color::Green;       // source hosts (green in screenshot)
const COLOR_HOST2: Color = Color::Green;       // dest hosts (green in screenshot)
const COLOR_PROC: Color = Color::Green;
const COLOR_ARROW: Color = Color::Blue;
const COLOR_2S: Color = Color::Yellow;         // 2s column
const COLOR_10S: Color = Color::Green;         // 10s column
const COLOR_40S: Color = Color::Cyan;          // 40s column
const COLOR_SCALE_LABEL: Color = Color::Green;
const COLOR_SCALE_LINE: Color = Color::Green;
const COLOR_TOTAL_LABEL: Color = Color::Blue;
const COLOR_CUM_LABEL: Color = Color::Yellow;
const COLOR_PEAK_LABEL: Color = Color::Magenta;
const COLOR_HEADER: Color = Color::Cyan;

/// Width of each rate column
const RATE_COL_W: usize = 9;
/// Width of the total column
const TOTAL_COL_W: usize = 8;
/// Number of rate columns (2s, 10s, 40s)
const RATE_COLS: usize = 3;
/// Total width for the right-side columns (total + 3 rates)
const RIGHT_AREA_W: usize = TOTAL_COL_W + RATE_COL_W * RATE_COLS;

pub fn draw(frame: &mut Frame, state: &AppState) {
    let size = frame.area();

    if state.show_help {
        draw_help(frame, size);
        return;
    }

    // Layout: scale labels (1), scale ticks (1), flows (fill), separator (1), totals (3)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // scale labels
            Constraint::Length(1), // scale tick line
            Constraint::Min(4),   // flow area
            Constraint::Length(1), // separator
            Constraint::Length(3), // totals (TX/RX/TOTAL)
        ])
        .split(size);

    draw_scale_labels(frame, chunks[0], state);
    draw_scale_ticks(frame, chunks[1], state);
    draw_flows(frame, chunks[2], state);
    draw_separator(frame, chunks[3]);
    draw_totals(frame, chunks[4], state);
}

// ─── Scale: fixed log10, 0 bps to 1 Gbps ─────────────────────────────────────
//
// Scale ticks in bits/sec: 10b, 1kb, 100kb, 10Mb, 1Gb
// log10 values:              1,   3,     5,    7,   9
// Positions (fraction):    1/9, 3/9,   5/9,  7/9, 9/9
// 1 Gbps is at the RIGHT EDGE of the screen.

/// log10 of max in bits = log10(1e9) = 9
const LOG10_MAX_BITS: f64 = 9.0;

/// Scale tick values in bytes/sec and their log10(bits) position.
const SCALE_TICKS: [(f64, f64); 5] = [
    (1.25,           1.0), // 10b      → log10(10)   = 1
    (125.0,          3.0), // 1kb      → log10(1000) = 3
    (12_500.0,       5.0), // 100kb    → log10(1e5)  = 5
    (1_250_000.0,    7.0), // 10Mb     → log10(1e7)  = 7
    (125_000_000.0,  9.0), // 1Gb      → log10(1e9)  = 9 (right edge)
];

/// Convert a byte rate to a screen fraction (0.0 – 1.0) using log10 scale.
/// 0 bytes → 0.0, 125_000_000 bytes (1Gbps) → 1.0
fn rate_to_frac(bytes_per_sec: f64) -> f64 {
    if bytes_per_sec <= 0.0 {
        return 0.0;
    }
    let bits = bytes_per_sec * 8.0;
    let log_val = bits.log10();
    (log_val / LOG10_MAX_BITS).clamp(0.0, 1.0)
}

/// Compute bar length in columns for a given byte rate.
fn bar_length(bytes_per_sec: f64, total_cols: u16) -> u16 {
    let frac = rate_to_frac(bytes_per_sec);
    (frac * total_cols as f64).round() as u16
}

fn draw_scale_labels(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.width < 20 {
        return;
    }
    let w = area.width as usize;
    let buf = frame.buffer_mut();
    let style = Style::default().fg(COLOR_SCALE_LABEL);

    for &(val_bytes, log_pos) in &SCALE_TICKS {
        let frac = log_pos / LOG10_MAX_BITS; // 1/9, 3/9, 5/9, 7/9, 9/9
        let tick_x = (frac * w as f64).round() as usize;
        let label = readable_size(val_bytes, state.use_bytes);
        let label = label.trim().to_string();
        let label_start = tick_x.saturating_sub(label.len() / 2);
        let x = area.x + (label_start as u16).min(area.width.saturating_sub(label.len() as u16));
        buf.set_string(x, area.y, &label, style);
    }
}

fn draw_scale_ticks(frame: &mut Frame, area: Rect, _state: &AppState) {
    if area.width < 10 {
        return;
    }
    let w = area.width as usize;
    let buf = frame.buffer_mut();
    let style = Style::default().fg(COLOR_SCALE_LINE);

    // Draw horizontal line
    for x in area.x..area.x + area.width {
        buf.set_string(x, area.y, "─", style);
    }

    // Corner at position 0
    buf.set_string(area.x, area.y, "└", style);

    // Tick marks at logarithmic positions matching the labels
    for &(_val, log_pos) in &SCALE_TICKS {
        let frac = log_pos / LOG10_MAX_BITS;
        let tick_x = ((frac * w as f64).round() as usize).min(w - 1);
        buf.set_string(area.x + tick_x as u16, area.y, "┴", style);
    }
}

fn draw_flows(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.height < 1 || area.width < 30 {
        return;
    }

    let w = area.width;

    // One line per flow pair
    let max_visible = area.height as usize;
    let start = state.scroll_offset.min(state.flows.len().saturating_sub(1));
    let visible = &state.flows[start..];
    let visible = &visible[..visible.len().min(max_visible)];

    // Host column width:
    // Layout: [src L] [" <=> " 5] [dst+proc L] [RIGHT_AREA_W on right]
    let host_area = (w as usize).saturating_sub(RIGHT_AREA_W + 5); // 5 for " <=> "
    let host_l = host_area / 2;
    let host_l = host_l.max(8).min(60);

    let buf = frame.buffer_mut();

    for (i, flow) in visible.iter().enumerate() {
        let y = area.y + i as u16;
        if y >= area.y + area.height {
            break;
        }

        let src_name = state.format_host(flow.key.src, flow.key.src_port, &flow.key.protocol);
        let dst_name = state.format_host(flow.key.dst, flow.key.dst_port, &flow.key.protocol);

        let proc_suffix = if state.show_processes {
            match (&flow.process_name, flow.pid) {
                (Some(name), Some(pid)) => format!(" [{}:{}]", pid, name),
                (Some(name), None) => format!(" [{}]", name),
                (None, Some(pid)) => format!(" [{}]", pid),
                _ => String::new(),
            }
        } else {
            String::new()
        };

        // Combined rate for the bar
        let combined_rate = flow.sent_2s + flow.recv_2s;
        let blen = bar_length(combined_rate, w);
        let bar_color = COLOR_BAR_SENT;

        // Paint bar background
        paint_bar_bg(buf, area.x, y, blen, w, bar_color);

        // Source hostname (left-aligned)
        let src_display = format!("{:<width$}", truncate_str(&src_name, host_l), width = host_l);
        write_over_bar(buf, area.x, y, &src_display,
            Style::default().fg(COLOR_HOST1).add_modifier(Modifier::BOLD),
            area.x, blen, bar_color);

        // Arrow " <=> "
        let arrow_x = area.x + host_l as u16;
        write_over_bar(buf, arrow_x, y, " <=> ",
            Style::default().fg(COLOR_ARROW).add_modifier(Modifier::BOLD),
            area.x, blen, bar_color);

        // Destination hostname + process name
        let dst_x = arrow_x + 5;
        if proc_suffix.is_empty() {
            let dst_display = truncate_str(&dst_name, host_l);
            write_over_bar(buf, dst_x, y, &dst_display,
                Style::default().fg(COLOR_HOST2).add_modifier(Modifier::BOLD),
                area.x, blen, bar_color);
        } else {
            let max_dst = host_l.saturating_sub(proc_suffix.len());
            let dst_display = truncate_str(&dst_name, max_dst);
            write_over_bar(buf, dst_x, y, &dst_display,
                Style::default().fg(COLOR_HOST2).add_modifier(Modifier::BOLD),
                area.x, blen, bar_color);
            let proc_x = dst_x + dst_display.len() as u16;
            let proc_display = truncate_str(&proc_suffix, host_l.saturating_sub(dst_display.len()));
            write_over_bar(buf, proc_x, y, &proc_display,
                Style::default().fg(COLOR_PROC).add_modifier(Modifier::BOLD),
                area.x, blen, bar_color);
        }

        // Total + rate columns (right side)
        let right_x = area.x + w - RIGHT_AREA_W as u16;
        let total = flow.total_sent + flow.total_recv;
        write_right_cols(buf, right_x, y, total,
            flow.sent_2s + flow.recv_2s,
            flow.sent_10s + flow.recv_10s,
            flow.sent_40s + flow.recv_40s,
            state.use_bytes, area.x, blen, bar_color);
    }
}

/// Paint bar background cells from area_x to area_x + bar_len.
fn paint_bar_bg(buf: &mut Buffer, area_x: u16, y: u16, bar_len: u16, total_w: u16, color: Color) {
    let buf_w = buf.area().width;
    let buf_x = buf.area().x;
    let buf_h = buf.area().height;
    let buf_y = buf.area().y;
    for x in area_x..area_x + bar_len.min(total_w) {
        if x < buf_x + buf_w && y < buf_y + buf_h {
            let cell = &mut buf[(x, y)];
            cell.set_char(' ');
            cell.set_bg(color);
        }
    }
}

/// Write text into the buffer, with bar background preserved for cells within the bar.
fn write_over_bar(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    text: &str,
    style: Style,
    area_x: u16,
    bar_len: u16,
    bar_color: Color,
) {
    let buf_max_x = buf.area().x + buf.area().width;
    let buf_max_y = buf.area().y + buf.area().height;
    for (i, ch) in text.chars().enumerate() {
        let cx = x + i as u16;
        if cx >= buf_max_x || y >= buf_max_y {
            break;
        }
        let cell = &mut buf[(cx, y)];
        cell.set_char(ch);
        if cx < area_x + bar_len {
            // On the bar: black text on colored background
            cell.set_fg(Color::Black);
            cell.set_bg(bar_color);
        } else {
            // Outside bar: normal colored text
            cell.set_fg(style.fg.unwrap_or(Color::Reset));
            cell.set_bg(Color::Reset);
            if style.add_modifier.contains(Modifier::BOLD) {
                cell.set_style(cell.style().add_modifier(Modifier::BOLD));
            }
        }
    }
}

/// Write the total + 3 rate columns at the given position.
fn write_right_cols(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    total: u64,
    r2s: f64,
    r10s: f64,
    r40s: f64,
    use_bytes: bool,
    area_x: u16,
    bar_len: u16,
    bar_color: Color,
) {
    let total_style = Style::default().fg(COLOR_CUM_LABEL);
    let s2 = Style::default().fg(COLOR_2S).add_modifier(Modifier::BOLD);
    let s10 = Style::default().fg(COLOR_10S).add_modifier(Modifier::BOLD);
    let s40 = Style::default().fg(COLOR_40S).add_modifier(Modifier::BOLD);

    let tt = format!("{:>7} ", readable_total(total, use_bytes));
    let t2 = format!("{:>8} ", readable_size(r2s, use_bytes));
    let t10 = format!("{:>8} ", readable_size(r10s, use_bytes));
    let t40 = format!("{:>8} ", readable_size(r40s, use_bytes));

    write_over_bar(buf, x, y, &tt, total_style, area_x, bar_len, bar_color);
    let rx = x + TOTAL_COL_W as u16;
    write_over_bar(buf, rx, y, &t2, s2, area_x, bar_len, bar_color);
    write_over_bar(buf, rx + RATE_COL_W as u16, y, &t10, s10, area_x, bar_len, bar_color);
    write_over_bar(buf, rx + (RATE_COL_W * 2) as u16, y, &t40, s40, area_x, bar_len, bar_color);
}

// ─── Separator ────────────────────────────────────────────────────────────────

fn draw_separator(frame: &mut Frame, area: Rect) {
    let buf = frame.buffer_mut();
    let style = Style::default().fg(COLOR_SCALE_LINE);
    for x in area.x..area.x + area.width {
        buf.set_string(x, area.y, "─", style);
    }
}

// ─── Totals ───────────────────────────────────────────────────────────────────

fn draw_totals(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.height < 3 {
        return;
    }

    let t = &state.totals;
    let buf = frame.buffer_mut();
    let right_x = area.x + (area.width as usize).saturating_sub(RIGHT_AREA_W) as u16;
    let rates_x = right_x + TOTAL_COL_W as u16;

    let label_style = Style::default()
        .fg(COLOR_TOTAL_LABEL)
        .add_modifier(Modifier::BOLD);
    let cum_style = Style::default().fg(COLOR_CUM_LABEL);
    let peak_style = Style::default()
        .fg(COLOR_PEAK_LABEL)
        .add_modifier(Modifier::BOLD);
    let s2 = Style::default().fg(COLOR_2S).add_modifier(Modifier::BOLD);
    let s10 = Style::default().fg(COLOR_10S).add_modifier(Modifier::BOLD);
    let s40 = Style::default().fg(COLOR_40S).add_modifier(Modifier::BOLD);

    // Helper to render a totals row
    let draw_row = |buf: &mut Buffer, y: u16, label: &str, cum: u64, peak: f64, r2s: f64, r10s: f64, r40s: f64| {
        buf.set_string(area.x, y, label, label_style);
        buf.set_string(area.x + 8, y, "cum:", cum_style);
        buf.set_string(
            area.x + 13,
            y,
            &format!("{:>8}", readable_total(cum, state.use_bytes)),
            cum_style,
        );
        buf.set_string(area.x + 23, y, "peak:", peak_style);
        buf.set_string(
            area.x + 29,
            y,
            &format!("{:>8}", readable_size(peak, state.use_bytes)),
            peak_style,
        );

        // "rates:" label
        let rates_label_x = rates_x.saturating_sub(8);
        buf.set_string(rates_label_x, y, "rates:", label_style);

        buf.set_string(rates_x, y, &format!("{:>8} ", readable_size(r2s, state.use_bytes)), s2);
        buf.set_string(rates_x + RATE_COL_W as u16, y, &format!("{:>8} ", readable_size(r10s, state.use_bytes)), s10);
        buf.set_string(rates_x + (RATE_COL_W * 2) as u16, y, &format!("{:>8} ", readable_size(r40s, state.use_bytes)), s40);
    };

    draw_row(buf, area.y, "TX:", t.cumulative_sent, t.peak_sent, t.sent_2s, t.sent_10s, t.sent_40s);
    draw_row(buf, area.y + 1, "RX:", t.cumulative_recv, t.peak_recv, t.recv_2s, t.recv_10s, t.recv_40s);
    draw_row(
        buf,
        area.y + 2,
        "TOTAL:",
        t.cumulative_sent + t.cumulative_recv,
        t.peak_sent + t.peak_recv,
        t.sent_2s + t.recv_2s,
        t.sent_10s + t.recv_10s,
        t.sent_40s + t.recv_40s,
    );
}

// ─── Help ─────────────────────────────────────────────────────────────────────

fn draw_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            " iftoprs - Keyboard Controls",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            " Display:",
            Style::default()
                .fg(COLOR_HEADER)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("   n     Toggle DNS resolution"),
        Line::from("   N     Toggle service name resolution"),
        Line::from("   t     Cycle line display mode"),
        Line::from("   p     Toggle port display"),
        Line::from("   Z     Toggle process display"),
        Line::from("   b     Toggle bar graphs"),
        Line::from("   B     Toggle bytes/bits"),
        Line::from("   T     Toggle cumulative totals"),
        Line::from("   P     Pause/resume display"),
        Line::from(""),
        Line::from(Span::styled(
            " Sorting:",
            Style::default()
                .fg(COLOR_HEADER)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("   1     Sort by 2s average"),
        Line::from("   2     Sort by 10s average"),
        Line::from("   3     Sort by 40s average"),
        Line::from("   <     Sort by source name"),
        Line::from("   >     Sort by destination name"),
        Line::from("   o     Freeze current sort order"),
        Line::from(""),
        Line::from(Span::styled(
            " Navigation:",
            Style::default()
                .fg(COLOR_HEADER)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("   j/Down  Scroll down"),
        Line::from("   k/Up    Scroll up"),
        Line::from(""),
        Line::from(Span::styled(
            " Other:",
            Style::default()
                .fg(COLOR_HEADER)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("   h     Toggle this help"),
        Line::from("   q     Quit"),
        Line::from(""),
        Line::from(Span::styled(
            " Press h to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(COLOR_HEADER))
        .title(Span::styled(
            " Help ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));

    let para = Paragraph::new(help_text).block(block);
    frame.render_widget(para, area);
}

// ─── Utilities ────────────────────────────────────────────────────────────────

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    if max_len <= 1 {
        return s.chars().next().map(|c| c.to_string()).unwrap_or_default();
    }
    let truncated: String = s.chars().take(max_len - 1).collect();
    format!("{}~", truncated)
}
