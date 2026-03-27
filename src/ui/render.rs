use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::Frame;

use crate::config::theme::{Theme, ThemeName};
use crate::ui::app::{AppState, BarStyle, ViewTab};
use crate::util::format::{readable_size, readable_total};

const RATE_COL_W: usize = 9;
const TOTAL_COL_W: usize = 8;
const RIGHT_AREA_W: usize = TOTAL_COL_W + RATE_COL_W * 3;
const PROC_COL_W: usize = 20;
const LOG10_MAX_BITS: f64 = 9.0;
const DIM_BORDER: Color = Color::Indexed(240);

const SCALE_TICKS: [(f64, f64); 5] = [
    (1.25, 1.0), (125.0, 3.0), (12_500.0, 5.0), (1_250_000.0, 7.0), (125_000_000.0, 9.0),
];

fn rate_to_frac(bps: f64) -> f64 {
    if bps <= 0.0 { return 0.0; }
    ((bps * 8.0).log10() / LOG10_MAX_BITS).clamp(0.0, 1.0)
}

fn bar_length(bps: f64, cols: u16) -> u16 {
    (rate_to_frac(bps) * cols as f64).round() as u16
}

// ─── Main draw ────────────────────────────────────────────────────────────────

pub fn draw(frame: &mut Frame, state: &mut AppState) {
    let size = frame.area();
    if state.show_help { draw_help(frame, size, state); return; }

    // Alert flash — override border color when flashing
    let is_flashing = state.alert_state.is_flashing();

    // Border support
    let border = state.show_border;
    let border_color = if is_flashing {
        Color::Indexed(196) // red flash
    } else if state.paused {
        DIM_BORDER
    } else {
        state.theme.scale_line
    };
    let margin: u16 = if border { 1 } else { 0 };

    if border {
        let buf = frame.buffer_mut();
        let bs = Style::default().fg(border_color);
        let x1 = size.width.saturating_sub(1);
        let y1 = size.height.saturating_sub(1);
        set_cell(buf, 0, 0, "┌", bs);
        set_cell(buf, x1, 0, "┐", bs);
        set_cell(buf, 0, y1, "└", bs);
        set_cell(buf, x1, y1, "┘", bs);
        for x in 1..x1 {
            set_cell(buf, x, 0, "─", bs);
            set_cell(buf, x, y1, "─", bs);
        }
        for y in 1..y1 {
            set_cell(buf, 0, y, "│", bs);
            set_cell(buf, x1, y, "│", bs);
        }

        // Title in top border
        let ver = env!("CARGO_PKG_VERSION");
        let title = if state.paused {
            format!(" ⏸ IFTOPRS v{} — PAUSED ", ver)
        } else {
            format!(" ▶▶▶ IFTOPRS v{} ◀◀◀ ", ver)
        };
        let title_chars = title.chars().count() as u16;
        let tx = (size.width.saturating_sub(title_chars)) / 2;
        let ts = if state.paused {
            Style::default().fg(Color::Indexed(196)).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(border_color).add_modifier(Modifier::BOLD)
        };
        set_str(buf, tx, 0, &title, ts, title_chars);
    }

    // Inner area (inside borders)
    let inner = Rect {
        x: margin,
        y: margin,
        width: size.width.saturating_sub(margin * 2),
        height: size.height.saturating_sub(margin * 2),
    };

    // Layout: scale + flows + separator + totals + optional header (bottom)
    let header_h = if state.show_header { 1 } else { 0 };
    let c = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Length(1), Constraint::Length(1), Constraint::Min(4),
        Constraint::Length(1), Constraint::Length(3),
        Constraint::Length(header_h),
    ]).split(inner);

    // Store layout positions for mouse hit-testing
    state.flow_area_y = c[2].y;
    state.header_bar_y = c[5].y;

    draw_scale_labels(frame, c[0], state);
    draw_scale_ticks(frame, c[1], state);
    match state.view_tab {
        ViewTab::Flows => draw_flows(frame, c[2], state, is_flashing),
        ViewTab::Processes => draw_processes(frame, c[2], state),
    }
    draw_separator(frame, c[3], state);
    draw_totals(frame, c[4], state);
    if state.show_header { draw_header(frame, c[5], state); }

    // Pause overlay
    if state.paused {
        draw_pause_overlay(frame, size, state);
    }

    // Overlays
    if state.theme_chooser.active { draw_theme_chooser(frame, size, state); }
    if state.theme_edit.active { draw_theme_editor(frame, size, state); }
    if state.interface_chooser.active { draw_interface_chooser(frame, size, state); }
    if state.filter_state.active { draw_filter_popup(frame, size, state); }
    if state.tooltip.active { draw_tooltip(frame, size, state); }

    // Hover tooltip on header bar segments
    if state.show_header && !state.show_help && !state.theme_chooser.active
        && !state.filter_state.active && state.hover.ready()
        && let Some((_, hy)) = state.hover.pos
        && hy == state.header_bar_y
    {
        draw_header_hover_tooltip(frame, size, state);
    }

    if let Some(ref msg) = state.status_msg
        && !msg.expired() { draw_status(frame, size, state, &msg.text); }
}

// ─── Header bar ──────────────────────────────────────────────────────────────

fn draw_header(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.height < 1 { return; }
    let buf = frame.buffer_mut();
    let t = &state.theme;
    let banner_s = Style::default().fg(t.scale_label).bg(Color::Indexed(236));
    let accent_s = Style::default().fg(t.host_src).bg(Color::Indexed(236)).add_modifier(Modifier::BOLD);
    let inner_w = area.width;

    // Fill background
    for x in area.x..area.x + inner_w {
        set_cell(buf, x, area.y, " ", Style::default().bg(Color::Indexed(236)));
    }

    let s = " │ ";
    let now = chrono::Local::now();
    let iface = if state.interface_name.is_empty() { "auto" } else { &state.interface_name };
    let sort_name = match state.sort_column {
        crate::ui::app::SortColumn::Avg2s => "2s",
        crate::ui::app::SortColumn::Avg10s => "10s",
        crate::ui::app::SortColumn::Avg40s => "40s",
        crate::ui::app::SortColumn::SrcName => "src",
        crate::ui::app::SortColumn::DstName => "dst",
    };
    let mut title = format!(
        " ▶▶▶ IFTOPRS ◀◀◀{s}iface:{}{s}flows:{}{s}clock:{}{s}sort:{}{s}rate:{}s{s}theme:{}",
        iface,
        state.total_flow_count,
        now.format("%H:%M:%S"),
        sort_name,
        state.refresh_rate,
        state.theme_name.display_name(),
    );
    if state.paused {
        title.push_str(&format!("{s}⏸ PAUSED"));
    }
    if let Some(ref filter) = state.screen_filter {
        title.push_str(&format!("{s}filter:{filter}"));
    }

    let help_hint = " │ h=help ";
    let help_hint_cw = help_hint.chars().count();
    let avail = inner_w as usize;
    let title_cw = title.chars().count();
    if title_cw + help_hint_cw < avail {
        let pad = avail - title_cw - help_hint_cw;
        title.push_str(&" ".repeat(pad));
        title.push_str(help_hint);
    }

    let title_display: String = title.chars().take(inner_w as usize).collect();
    set_str(buf, area.x, area.y, &title_display, banner_s, inner_w);

    // Highlight "IFTOPRS" in accent color
    if let Some(idx) = title_display.find("IFTOPRS") {
        let char_offset = title_display[..idx].chars().count() as u16;
        set_str(buf, area.x + char_offset, area.y, "IFTOPRS", accent_s, 7);
    }
}

// ─── Scale ────────────────────────────────────────────────────────────────────

fn draw_scale_labels(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.width < 20 { return; }
    let w = area.width as usize;
    let buf = frame.buffer_mut();
    let s = Style::default().fg(state.theme.scale_label);
    for &(v, lp) in &SCALE_TICKS {
        let tx = (lp / LOG10_MAX_BITS * w as f64).round() as usize;
        let l = readable_size(v, state.use_bytes); let l = l.trim();
        let lx = tx.saturating_sub(l.len() / 2);
        let x = area.x + (lx as u16).min(area.width.saturating_sub(l.len() as u16));
        buf.set_string(x, area.y, l, s);
    }
}

fn draw_scale_ticks(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.width < 10 { return; }
    let w = area.width as usize;
    let buf = frame.buffer_mut();
    let s = Style::default().fg(state.theme.scale_line);
    for x in area.x..area.x + area.width { buf.set_string(x, area.y, "─", s); }
    buf.set_string(area.x, area.y, "└", s);
    for &(_, lp) in &SCALE_TICKS {
        let tx = ((lp / LOG10_MAX_BITS * w as f64).round() as usize).min(w - 1);
        buf.set_string(area.x + tx as u16, area.y, "┴", s);
    }
}

// ─── Flows ────────────────────────────────────────────────────────────────────

fn draw_flows(frame: &mut Frame, area: Rect, state: &AppState, is_flashing: bool) {
    if area.height < 1 || area.width < 30 || state.flows.is_empty() { return; }
    let t = &state.theme;
    let w = area.width;
    let start = state.scroll_offset.min(state.flows.len() - 1);
    let vis = &state.flows[start..];
    let vis = &vis[..vis.len().min(area.height as usize)];

    // Layout: [src hl] [ <=> 5] [dst hl] [proc PROC_COL_W] [RIGHT_AREA_W]
    let proc_w: usize = if state.show_processes { PROC_COL_W } else { 0 };
    let ha = (w as usize).saturating_sub(RIGHT_AREA_W + proc_w + 5);
    let hl = (ha / 2).clamp(8, 60);
    let buf = frame.buffer_mut();

    for (i, f) in vis.iter().enumerate() {
        let y = area.y + i as u16;
        if y >= area.y + area.height { break; }
        let flow_idx = start + i;
        let is_selected = state.selected == Some(flow_idx);
        let is_pinned = state.is_pinned(&f.key);

        let src = state.format_host(f.key.src, f.key.src_port, &f.key.protocol);
        let dst = state.format_host(f.key.dst, f.key.dst_port, &f.key.protocol);

        let rate = f.sent_2s + f.recv_2s;
        let bl = bar_length(rate, w);
        let bs = state.bar_style;

        // Alert flash background for flows above threshold
        if is_flashing && state.alert_threshold > 0.0 && rate >= state.alert_threshold {
            let flash_bg = Style::default().bg(Color::Indexed(52));
            for x in area.x..area.x + w { set_cell(buf, x, y, " ", flash_bg); }
        }

        // Bar first
        paint_bar_styled(buf, area.x, y, bl, w, t.bar_color, bs);

        // Pin indicator
        let pin_prefix = if is_pinned { "★ " } else { "" };
        let src_with_pin = format!("{}{}", pin_prefix, src);

        // src hostname
        let sd = format!("{:<w$}", trunc(&src_with_pin, hl), w = hl);
        write_bar_styled(buf, area.x, y, &sd, t.host_src, area.x, bl, t.bar_color, t.bar_text, bs);

        // <=>
        let ax = area.x + hl as u16;
        write_bar_styled(buf, ax, y, " <=> ", t.arrow, area.x, bl, t.bar_color, t.bar_text, bs);

        // dst hostname
        let dx = ax + 5;
        let dd = format!("{:<w$}", trunc(&dst, hl), w = hl);
        write_bar_styled(buf, dx, y, &dd, t.host_dst, area.x, bl, t.bar_color, t.bar_text, bs);

        // process column
        if state.show_processes && proc_w > 0 {
            let proc_x = dx + hl as u16;
            let proc_s = match (&f.process_name, f.pid) {
                (Some(n), Some(p)) => format!("[{}:{}]", p, n),
                (Some(n), None) => format!("[{}]", n),
                (None, Some(p)) => format!("[{}]", p),
                _ => String::new(),
            };
            let pt = format!("{:>w$}", trunc(&proc_s, proc_w), w = proc_w);
            write_bar_styled(buf, proc_x, y, &pt, t.proc_name, area.x, bl, t.bar_color, t.bar_text, bs);
        }

        // right cols
        let rx = area.x + w - RIGHT_AREA_W as u16;
        write_right_styled(buf, rx, y, f.total_sent + f.total_recv,
            f.sent_2s + f.recv_2s, f.sent_10s + f.recv_10s, f.sent_40s + f.recv_40s,
            state.use_bytes, area.x, bl, t, bs);

        // Selection indicator — applied AFTER everything else so it's visible
        if is_selected {
            let buf_w = buf.area().width;
            let buf_x = buf.area().x;
            let buf_h = buf.area().height;
            let buf_y = buf.area().y;
            // Add underline to entire row + bright cursor at start
            for x in area.x..area.x + w {
                if x < buf_x + buf_w && y < buf_y + buf_h {
                    let c = &mut buf[(x, y)];
                    c.set_style(c.style().add_modifier(Modifier::UNDERLINED));
                }
            }
            // Bright arrow indicator at column 0
            if area.x < buf_x + buf_w && y < buf_y + buf_h {
                let c = &mut buf[(area.x, y)];
                c.set_char('▶');
                c.set_fg(t.rate_2s);
                c.set_style(c.style().add_modifier(Modifier::BOLD).remove_modifier(Modifier::UNDERLINED));
            }
        }
    }
}

// ─── Process aggregation view ─────────────────────────────────────────────────

fn draw_processes(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.height < 2 || area.width < 30 || state.process_snapshots.is_empty() { return; }
    let t = &state.theme;
    let w = area.width;
    let buf = frame.buffer_mut();

    // Header row
    let proc_name_w = 24usize;
    let flows_w = 8usize;
    let header = format!(
        " {:<pw$} {:>fw$} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9}",
        "PROCESS", "FLOWS", "TX 2s", "RX 2s", "TX 10s", "RX 10s", "TOTAL TX", "TOTAL RX",
        pw = proc_name_w, fw = flows_w,
    );
    let header_s = Style::default().fg(t.scale_label).add_modifier(Modifier::BOLD);
    let header_display: String = header.chars().take(w as usize).collect();
    set_str(buf, area.x, area.y, &header_display, header_s, w);

    let start = state.process_scroll.min(state.process_snapshots.len().saturating_sub(1));
    let vis = &state.process_snapshots[start..];
    let rows_available = (area.height - 1) as usize; // -1 for header
    let vis = &vis[..vis.len().min(rows_available)];

    let bs = state.bar_style;

    for (i, p) in vis.iter().enumerate() {
        let y = area.y + 1 + i as u16;
        if y >= area.y + area.height { break; }
        let proc_idx = start + i;
        let is_selected = state.process_selected == Some(proc_idx);

        let rate = p.sent_2s + p.recv_2s;
        let bl = bar_length(rate, w);

        // Bar background
        paint_bar_styled(buf, area.x, y, bl, w, t.bar_color, bs);

        // Process name (with PID)
        let name_display = match p.pid {
            Some(pid) => format!(" [{}] {}", pid, p.name),
            None => format!(" {}", p.name),
        };
        let name_trunc = format!("{:<w$}", trunc(&name_display, proc_name_w + 2), w = proc_name_w + 2);
        write_bar_styled(buf, area.x, y, &name_trunc, t.host_src, area.x, bl, t.bar_color, t.bar_text, bs);

        // Flow count
        let flows_str = format!("{:>fw$}", p.flow_count, fw = flows_w);
        let fx = area.x + proc_name_w as u16 + 2;
        write_bar_styled(buf, fx, y, &flows_str, t.proc_name, area.x, bl, t.bar_color, t.bar_text, bs);

        // Rate columns
        let cols_x = fx + flows_w as u16 + 1;
        let tx_2s = format!("{:>9}", readable_size(p.sent_2s, state.use_bytes));
        let rx_2s = format!("{:>9}", readable_size(p.recv_2s, state.use_bytes));
        let tx_10s = format!("{:>9}", readable_size(p.sent_10s, state.use_bytes));
        let rx_10s = format!("{:>9}", readable_size(p.recv_10s, state.use_bytes));
        let tot_tx = format!("{:>9}", readable_total(p.total_sent, state.use_bytes));
        let tot_rx = format!("{:>9}", readable_total(p.total_recv, state.use_bytes));

        write_bar_styled(buf, cols_x, y, &tx_2s, t.rate_2s, area.x, bl, t.bar_color, t.bar_text, bs);
        write_bar_styled(buf, cols_x + 10, y, &rx_2s, t.rate_2s, area.x, bl, t.bar_color, t.bar_text, bs);
        write_bar_styled(buf, cols_x + 20, y, &tx_10s, t.rate_10s, area.x, bl, t.bar_color, t.bar_text, bs);
        write_bar_styled(buf, cols_x + 30, y, &rx_10s, t.rate_10s, area.x, bl, t.bar_color, t.bar_text, bs);
        write_bar_styled(buf, cols_x + 40, y, &tot_tx, t.cum_label, area.x, bl, t.bar_color, t.bar_text, bs);
        write_bar_styled(buf, cols_x + 50, y, &tot_rx, t.cum_label, area.x, bl, t.bar_color, t.bar_text, bs);

        // Selection highlight
        if is_selected {
            let buf_w = buf.area().width;
            let buf_x = buf.area().x;
            let buf_h = buf.area().height;
            let buf_y = buf.area().y;
            for x in area.x..area.x + w {
                if x < buf_x + buf_w && y < buf_y + buf_h {
                    let c = &mut buf[(x, y)];
                    c.set_style(c.style().add_modifier(Modifier::UNDERLINED));
                }
            }
            if area.x < buf_x + buf_w && y < buf_y + buf_h {
                let c = &mut buf[(area.x, y)];
                c.set_char('▶');
                c.set_fg(t.rate_2s);
                c.set_style(c.style().add_modifier(Modifier::BOLD).remove_modifier(Modifier::UNDERLINED));
            }
        }
    }
}

// ─── Bottom totals with bars ──────────────────────────────────────────────────

fn draw_separator(frame: &mut Frame, area: Rect, state: &AppState) {
    let buf = frame.buffer_mut();
    let s = Style::default().fg(state.theme.scale_line);
    for x in area.x..area.x + area.width { buf.set_string(x, area.y, "─", s); }

    // Tab indicator on the left
    let tab_indicator = match state.view_tab {
        ViewTab::Flows => " [Flows] Processes ",
        ViewTab::Processes => " Flows [Processes] ",
    };
    let tab_s = Style::default().fg(state.theme.host_src).add_modifier(Modifier::BOLD);
    set_str(buf, area.x + 1, area.y, tab_indicator, tab_s, tab_indicator.len() as u16);
    let tab_hint_s = Style::default().fg(Color::Indexed(240));
    set_str(buf, area.x + 1 + tab_indicator.len() as u16, area.y, "Tab", tab_hint_s, 3);

    // Show interface name, flow count, refresh rate, and theme in separator
    let mut parts: Vec<String> = Vec::new();
    if !state.interface_name.is_empty() {
        parts.push(format!("iface:{}", state.interface_name));
    }
    parts.push(format!("flows:{}", state.flows.len()));
    parts.push(format!("rate:{}s", state.refresh_rate));
    parts.push(state.theme_name.display_name().to_string());
    if state.paused { parts.push("⏸".to_string()); }

    let info = format!(" {} ", parts.join(" │ "));
    let info_x = area.x + area.width.saturating_sub(info.len() as u16 + 1);
    let info_s = Style::default().fg(state.theme.cum_label);
    set_str(buf, info_x, area.y, &info, info_s, info.len() as u16);
}

fn draw_totals(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.height < 3 { return; }
    let th = &state.theme;
    let tot = &state.totals;
    let buf = frame.buffer_mut();
    let w = area.width;
    let rx = area.x + (w as usize).saturating_sub(RIGHT_AREA_W) as u16;
    let rrx = rx + TOTAL_COL_W as u16;

    let rows: [(u16, &str, u64, f64, f64, f64, f64); 3] = [
        (area.y, "TX:", tot.cumulative_sent, tot.peak_sent, tot.sent_2s, tot.sent_10s, tot.sent_40s),
        (area.y + 1, "RX:", tot.cumulative_recv, tot.peak_recv, tot.recv_2s, tot.recv_10s, tot.recv_40s),
        (area.y + 2, "TOTAL:", tot.cumulative_sent + tot.cumulative_recv,
            tot.peak_sent + tot.peak_recv,
            tot.sent_2s + tot.recv_2s, tot.sent_10s + tot.recv_10s, tot.sent_40s + tot.recv_40s),
    ];

    let bs = state.bar_style;
    for &(y, label, cum, peak, r2, r10, r40) in &rows {
        let bl = bar_length(r2, w);
        paint_bar_styled(buf, area.x, y, bl, w, th.bar_color, bs);

        write_bar_styled(buf, area.x, y, label, th.total_label, area.x, bl, th.bar_color, th.bar_text, bs);
        let cum_text = format!("  cum:{:>8}", readable_total(cum, state.use_bytes));
        write_bar_styled(buf, area.x + 8, y, &cum_text, th.cum_label, area.x, bl, th.bar_color, th.bar_text, bs);
        let peak_text = format!("  peak:{:>8}", readable_size(peak, state.use_bytes));
        write_bar_styled(buf, area.x + 24, y, &peak_text, th.peak_label, area.x, bl, th.bar_color, th.bar_text, bs);

        let rl_x = rrx.saturating_sub(8);
        write_bar_styled(buf, rl_x, y, "rates:", th.total_label, area.x, bl, th.bar_color, th.bar_text, bs);
        write_bar_styled(buf, rrx, y, &format!("{:>8} ", readable_size(r2, state.use_bytes)), th.rate_2s, area.x, bl, th.bar_color, th.bar_text, bs);
        write_bar_styled(buf, rrx + RATE_COL_W as u16, y, &format!("{:>8} ", readable_size(r10, state.use_bytes)), th.rate_10s, area.x, bl, th.bar_color, th.bar_text, bs);
        write_bar_styled(buf, rrx + (RATE_COL_W * 2) as u16, y, &format!("{:>8} ", readable_size(r40, state.use_bytes)), th.rate_40s, area.x, bl, th.bar_color, th.bar_text, bs);
    }
}

// ─── Buffer helpers ───────────────────────────────────────────────────────────

/// Paint the bar background across the full screen width.
/// For Solid: colored bg in bar region, nothing outside.
/// For others: colored bg in bar region (dimmer shade), nothing outside.
/// Text is overlaid on top by write_bar_styled.
fn paint_bar_styled(buf: &mut Buffer, x0: u16, y: u16, len: u16, max_w: u16, color: Color, _style: BarStyle) {
    // All styles use background color for the bar region — the style
    // only affects how write_bar_styled renders text on top.
    let bw = buf.area().width; let bx = buf.area().x; let bh = buf.area().height; let by = buf.area().y;
    let bar_len = len.min(max_w);
    for x in x0..x0 + bar_len {
        if x >= bx + bw || y >= by + bh { break; }
        let c = &mut buf[(x, y)];
        c.set_char(' ');
        c.set_bg(color);
    }
}

#[allow(clippy::too_many_arguments)]
fn write_bar_styled(buf: &mut Buffer, x: u16, y: u16, text: &str, fg: Color,
    x0: u16, bl: u16, bar_bg: Color, bar_fg: Color, _bs: BarStyle)
{
    let mx = buf.area().x + buf.area().width;
    let my = buf.area().y + buf.area().height;
    for (i, ch) in text.chars().enumerate() {
        let cx = x + i as u16;
        if cx >= mx || y >= my { break; }
        let c = &mut buf[(cx, y)];
        c.set_char(ch);
        if cx < x0 + bl {
            // Inside bar region — always black text on colored background
            c.set_fg(bar_fg); c.set_bg(bar_bg);
        } else {
            // Outside bar
            c.set_fg(fg); c.set_bg(Color::Reset);
            c.set_style(c.style().add_modifier(Modifier::BOLD));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn write_right_styled(buf: &mut Buffer, x: u16, y: u16, tot: u64, r2: f64, r10: f64, r40: f64,
    ub: bool, x0: u16, bl: u16, t: &Theme, bs: BarStyle)
{
    let tt = format!("{:>7} ", readable_total(tot, ub));
    write_bar_styled(buf, x, y, &tt, t.cum_label, x0, bl, t.bar_color, t.bar_text, bs);
    let rx = x + TOTAL_COL_W as u16;
    write_bar_styled(buf, rx, y, &format!("{:>8} ", readable_size(r2, ub)), t.rate_2s, x0, bl, t.bar_color, t.bar_text, bs);
    write_bar_styled(buf, rx + RATE_COL_W as u16, y, &format!("{:>8} ", readable_size(r10, ub)), t.rate_10s, x0, bl, t.bar_color, t.bar_text, bs);
    write_bar_styled(buf, rx + (RATE_COL_W * 2) as u16, y, &format!("{:>8} ", readable_size(r40, ub)), t.rate_40s, x0, bl, t.bar_color, t.bar_text, bs);
}

fn set_cell(buf: &mut Buffer, x: u16, y: u16, ch: &str, s: Style) {
    let a = buf.area();
    if x < a.x + a.width && y < a.y + a.height {
        let c = &mut buf[(x, y)];
        c.set_symbol(ch);
        c.set_style(s);
    }
}

fn set_str(buf: &mut Buffer, x: u16, y: u16, s: &str, st: Style, mw: u16) {
    let aw = buf.area().x + buf.area().width;
    let ah = buf.area().y + buf.area().height;
    if y >= ah { return; }
    let mut char_buf = [0u8; 4];
    for (i, ch) in s.chars().enumerate() {
        let cx = x + i as u16;
        if cx >= x + mw || cx >= aw { break; }
        let c = &mut buf[(cx, y)];
        c.set_symbol(ch.encode_utf8(&mut char_buf));
        c.set_style(st);
    }
}

/// Draw a filled box with double-line border. Returns (x0, y0, bw, bh).
fn draw_box(buf: &mut Buffer, area: Rect, bw: u16, bh: u16, bg: Color, border_style: Style) -> (u16, u16) {
    let x0 = (area.width.saturating_sub(bw)) / 2;
    let y0 = (area.height.saturating_sub(bh)) / 2;
    let x1 = x0 + bw - 1;
    let y1 = y0 + bh - 1;
    let fill = Style::default().bg(bg);
    for y in y0..y0 + bh { for x in x0..x0 + bw { set_cell(buf, x, y, " ", fill); } }
    set_cell(buf, x0, y0, "╔", border_style);
    set_cell(buf, x1, y0, "╗", border_style);
    set_cell(buf, x0, y1, "╚", border_style);
    set_cell(buf, x1, y1, "╝", border_style);
    for x in x0 + 1..x1 { set_cell(buf, x, y0, "═", border_style); set_cell(buf, x, y1, "═", border_style); }
    for y in y0 + 1..y1 { set_cell(buf, x0, y, "║", border_style); set_cell(buf, x1, y, "║", border_style); }
    (x0, y0)
}

fn trunc(s: &str, m: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= m { s.to_string() }
    else if m <= 1 { s.chars().next().map(|c| c.to_string()).unwrap_or_default() }
    else { let t: String = s.chars().take(m - 1).collect(); format!("{}~", t) }
}

// ─── Help modal (storageshower-style) ─────────────────────────────────────────

fn draw_help(frame: &mut Frame, area: Rect, state: &AppState) {
    let t = &state.theme;
    let buf = frame.buffer_mut();
    let bw = 90u16.min(area.width.saturating_sub(4));
    let bh = 31u16.min(area.height.saturating_sub(4));
    let bg = t.help_bg;
    let bs = Style::default().fg(t.help_border);
    let bgs = Style::default().fg(Color::White).bg(bg);
    let ks = Style::default().fg(t.help_key).bg(bg);
    let ts = Style::default().fg(t.help_title).bg(bg).add_modifier(Modifier::BOLD);
    let ss = Style::default().fg(t.help_section).bg(bg).add_modifier(Modifier::BOLD);

    let (x0, y0) = draw_box(buf, area, bw, bh, bg, bs);

    let ver = env!("CARGO_PKG_VERSION");
    let title = format!("⌨ IFTOPRS v{} — KEYBOARD SHORTCUTS", ver);
    let title_cw = title.chars().count() as u16;
    set_str(buf, x0 + (bw.saturating_sub(title_cw)) / 2, y0 + 1, &title, ts, bw - 2);
    let byline = "by MenkeTechnologies";
    let byline_s = Style::default().fg(Color::Indexed(240)).bg(bg);
    set_str(buf, x0 + (bw.saturating_sub(byline.len() as u16)) / 2, y0 + 2, byline, byline_s, bw - 2);
    let bl = "[ jacking into your packet stream ]";
    set_str(buf, x0 + (bw.saturating_sub(bl.len() as u16)) / 2, y0 + 3, bl, Style::default().fg(Color::Indexed(240)).bg(bg), bw - 2);

    let entries: [(&str, &[(&str, &str)]); 7] = [
        ("CAPTURE", &[("n","DNS toggle"),("N","Port names"),("p","Ports"),("Z","Processes"),("B","Bytes/bits"),("b","Bar style"),("T","Cumulative"),("P","Pause")]),
        ("SORT", &[("1","By 2s"),("2","By 10s"),("3","By 40s"),("<","By source"),(">","By dest"),("r","Reverse"),("o","Freeze order")]),
        ("NAV", &[("j/↓","Select next"),("k/↑","Select prev"),("^D","Half-page dn"),("^U","Half-page up"),("G/End","Jump last"),("Home","Jump first"),("Esc","Deselect")]),
        ("FILTER", &[("/","Search flows"),("0","Clear filter")]),
        ("ACTIONS", &[("e","Export flows"),("y","Copy selected"),("F","Pin/unpin ★")]),
        ("DISPLAY", &[("Tab","Switch view"),("c","Theme chooser"),("C","Theme editor"),("i","Interface"),("t","Line mode"),("x","Toggle border"),("g","Toggle header"),("f","Refresh rate"),("h/?","Toggle help"),("q","Quit")]),
        ("", &[]),
    ];

    let cw = ((bw as usize).saturating_sub(4)) / 3;
    let mut col = 0usize;
    let mut row = 0usize;
    for (section, keys) in &entries {
        if section.is_empty() { continue; }
        if row + keys.len() + 2 > (bh as usize - 6) { col += 1; row = 0; if col >= 3 { break; } }
        let cx = x0 + 2 + (col as u16) * cw as u16;
        let sy = y0 + 5 + row as u16;
        set_str(buf, cx, sy, section, ss, cw as u16);
        row += 1;
        for &(k, d) in *keys {
            let ey = y0 + 5 + row as u16;
            if ey >= y0 + bh - 2 { break; }
            set_str(buf, cx, ey, k, ks, 8);
            set_str(buf, cx + 9, ey, d, bgs, 18);
            row += 1;
        }
        row += 1;
    }

    let tl = format!("theme: {} | c=chooser", state.theme_name.display_name());
    set_str(buf, x0 + (bw.saturating_sub(tl.len() as u16)) / 2, y0 + bh - 3,
        &tl, Style::default().fg(t.help_val).bg(bg), bw - 4);
    set_str(buf, x0 + (bw.saturating_sub(16)) / 2, y0 + bh - 2,
        "press h to close", Style::default().fg(Color::Indexed(240)).bg(bg), bw - 4);
}

// ─── Theme editor ─────────────────────────────────────────────────────────────

fn draw_theme_editor(frame: &mut Frame, area: Rect, state: &AppState) {
    let t = &state.theme;
    let te = &state.theme_edit;
    let buf = frame.buffer_mut();
    let bw = 56u16.min(area.width.saturating_sub(4));
    let bh: u16 = if te.naming { 16 } else { 15 };
    let bh = bh.min(area.height.saturating_sub(4));
    let bg = t.help_bg;
    let bs = Style::default().fg(t.help_border);
    let bgs = Style::default().fg(Color::White).bg(bg);
    let ts = Style::default().fg(t.help_title).bg(bg).add_modifier(Modifier::BOLD);
    let hint_s = Style::default().fg(Color::Indexed(240)).bg(bg);
    let sel_s = Style::default().fg(Color::White).bg(Color::Indexed(237));

    let (x0, y0) = draw_box(buf, area, bw, bh, bg, bs);

    // Title
    let title = "\u{1F3A8} THEME EDITOR";
    let tlen = title.chars().count() as u16;
    set_str(buf, x0 + (bw.saturating_sub(tlen)) / 2, y0 + 1, title, ts, bw - 2);

    // Color channel labels
    let labels = ["primary", "accent", "c3", "c4", "c5", "c6"];
    let colors = te.colors;

    for (i, label) in labels.iter().enumerate() {
        let row_y = y0 + 3 + i as u16;
        if row_y >= y0 + bh - 2 { break; }
        let is_sel = i == te.slot;

        let row_style = if is_sel { sel_s } else { bgs };
        if is_sel {
            for x in x0 + 1..x0 + bw - 1 {
                set_cell(buf, x, row_y, " ", sel_s);
            }
        }

        let marker = if is_sel { "\u{25B8} " } else { "  " };
        set_str(buf, x0 + 2, row_y, marker, row_style, 2);

        let label_str = format!("{:<10}", label);
        set_str(buf, x0 + 4, row_y, &label_str, row_style, 10);

        let val_str = format!("{:>3}", colors[i]);
        set_str(buf, x0 + 15, row_y, &val_str, row_style, 3);

        // Color swatch
        let swatch_s = Style::default().fg(Color::Indexed(colors[i])).bg(bg);
        set_str(buf, x0 + 20, row_y, "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}", swatch_s, 5);

        // Arrow preview
        let arrow_s = Style::default().fg(Color::Indexed(colors[i])).bg(bg);
        set_str(buf, x0 + 26, row_y, " \u{25C0}\u{2500}\u{2500}\u{25B6}", arrow_s, 5);
    }

    // Preview bar using the full palette
    let preview_y = y0 + 10;
    if preview_y < y0 + bh - 2 {
        set_str(buf, x0 + 2, preview_y, "preview:", hint_s, 8);
        let preview_w = (bw as usize).saturating_sub(13);
        for j in 0..preview_w {
            let frac = j as f64 / preview_w as f64;
            let c = if frac < 0.20 {
                Color::Indexed(colors[0]) // primary
            } else if frac < 0.40 {
                Color::Indexed(colors[1]) // accent
            } else if frac < 0.55 {
                Color::Indexed(colors[2]) // c3
            } else if frac < 0.70 {
                Color::Indexed(colors[3]) // c4
            } else if frac < 0.85 {
                Color::Indexed(colors[4]) // c5
            } else {
                Color::Indexed(colors[5]) // c6
            };
            set_cell(buf, x0 + 11 + j as u16, preview_y, "\u{2588}", Style::default().fg(c).bg(bg));
        }
    }

    // Naming prompt or keybind hints
    if te.naming {
        let name_y = y0 + 12;
        if name_y < y0 + bh - 1 {
            let input_s = Style::default().fg(Color::Indexed(48)).bg(Color::Indexed(235));
            set_str(buf, x0 + 2, name_y, "Theme name:", bgs, 11);
            let name_display = format!("{}_", te.name);
            set_str(buf, x0 + 14, name_y, &name_display, input_s, bw - 16);
            set_str(buf, x0 + 2, name_y + 1, "Enter:save  Esc:back", hint_s, bw - 4);
        }
    } else {
        let hint_y = y0 + 12;
        if hint_y < y0 + bh - 1 {
            set_str(buf, x0 + 2, hint_y, "j/k:select  h/l:\u{00B1}1  H/L:\u{00B1}10", hint_s, bw - 4);
            set_str(buf, x0 + 2, hint_y + 1, "Enter/s:save  Esc/q:cancel", hint_s, bw - 4);
        }
    }
}

// ─── Theme chooser ────────────────────────────────────────────────────────────

fn draw_theme_chooser(frame: &mut Frame, area: Rect, state: &AppState) {
    let t = &state.theme;
    let ch = &state.theme_chooser;
    let buf = frame.buffer_mut();
    let bw = 50u16.min(area.width.saturating_sub(4));
    let bh = (ThemeName::ALL.len() as u16 + 6).min(area.height.saturating_sub(4));
    let bg = t.help_bg;
    let bs = Style::default().fg(t.help_border);
    let ts = Style::default().fg(t.help_title).bg(bg).add_modifier(Modifier::BOLD);

    let (x0, y0) = draw_box(buf, area, bw, bh, bg, bs);
    set_str(buf, x0 + 2, y0 + 1, "THEME CHOOSER", ts, bw - 4);

    for (i, &tn) in ThemeName::ALL.iter().enumerate() {
        let ey = y0 + 3 + i as u16;
        if ey >= y0 + bh - 2 { break; }
        let sel = i == ch.selected;
        let act = tn == state.theme_name;
        let mk = if act { "▸ " } else { "  " };
        let rs = if sel { Style::default().fg(Color::Black).bg(t.help_key) }
                 else { Style::default().fg(Color::White).bg(bg) };
        set_str(buf, x0 + 2, ey, &format!("{}{:<20}", mk, tn.display_name()), rs, 24);
        let swatch = Theme::swatch(tn);
        let sx = x0 + 26;
        for (si, (color, block)) in swatch.iter().enumerate() {
            let ss = if sel { Style::default().fg(*color).bg(t.help_key) }
                     else { Style::default().fg(*color).bg(bg) };
            set_str(buf, sx + (si as u16) * 2, ey, block, ss, 2);
        }
    }

    let ft = "j/k:nav  Enter:select  Esc:cancel";
    set_str(buf, x0 + (bw.saturating_sub(ft.len() as u16)) / 2, y0 + bh - 2,
        ft, Style::default().fg(Color::Indexed(240)).bg(bg), bw - 4);
}

// ─── Interface chooser ───────────────────────────────────────────────────────

fn draw_interface_chooser(frame: &mut Frame, area: Rect, state: &AppState) {
    let t = &state.theme;
    let ch = &state.interface_chooser;
    let buf = frame.buffer_mut();
    let bw = 50u16.min(area.width.saturating_sub(4));
    let bh = (ch.interfaces.len() as u16 + 5).min(area.height.saturating_sub(4));
    let (x0, y0) = draw_box(buf, area, bw, bh, t.help_bg, Style::default().fg(t.help_border));
    let bg = t.help_bg;
    let ts = Style::default().fg(t.help_title).bg(bg).add_modifier(Modifier::BOLD);

    set_str(buf, x0 + 2, y0 + 1, "INTERFACE CHOOSER", ts, bw - 4);

    for (i, iface) in ch.interfaces.iter().enumerate() {
        let ey = y0 + 3 + i as u16;
        if ey >= y0 + bh - 2 { break; }
        let sel = i == ch.selected;
        let act = *iface == state.interface_name;
        let mk = if act { "▸ " } else { "  " };
        let rs = if sel { Style::default().fg(Color::Black).bg(t.help_key) }
                 else { Style::default().fg(Color::White).bg(bg) };
        set_str(buf, x0 + 2, ey, &format!("{}{}", mk, iface), rs, bw - 4);
    }

    let ft = "j/k:nav  i:next  Enter:select  Esc:cancel";
    set_str(buf, x0 + (bw.saturating_sub(ft.len() as u16)) / 2, y0 + bh - 2,
        ft, Style::default().fg(Color::Indexed(240)).bg(bg), bw - 4);
}

// ─── Filter popup ─────────────────────────────────────────────────────────────

fn draw_filter_popup(frame: &mut Frame, area: Rect, state: &AppState) {
    let t = &state.theme;
    let fs = &state.filter_state;
    let buf = frame.buffer_mut();
    let bw = 50u16.min(area.width.saturating_sub(4));
    let bh = 5u16;
    let x0 = (area.width.saturating_sub(bw)) / 2;
    let y0 = (area.height.saturating_sub(bh)) / 2;
    let bg = t.help_bg;
    let bs = Style::default().fg(t.help_border);

    // Fill + border
    for y in y0..y0 + bh { for x in x0..x0 + bw { set_cell(buf, x, y, " ", Style::default().bg(bg)); } }
    set_cell(buf, x0, y0, "╭", bs); set_cell(buf, x0 + bw - 1, y0, "╮", bs);
    set_cell(buf, x0, y0 + bh - 1, "╰", bs); set_cell(buf, x0 + bw - 1, y0 + bh - 1, "╯", bs);
    for x in x0 + 1..x0 + bw - 1 { set_cell(buf, x, y0, "─", bs); set_cell(buf, x, y0 + bh - 1, "─", bs); }
    for y in y0 + 1..y0 + bh - 1 { set_cell(buf, x0, y, "│", bs); set_cell(buf, x0 + bw - 1, y, "│", bs); }

    let ts = Style::default().fg(t.help_title).bg(bg).add_modifier(Modifier::BOLD);
    set_str(buf, x0 + 2, y0 + 1, "Search:", ts, 8);

    // Input buffer with cursor
    let input_x = x0 + 10;
    let input_w = (bw - 12) as usize;
    let display = if fs.buf.len() > input_w { &fs.buf[fs.buf.len() - input_w..] } else { &fs.buf };
    let is = Style::default().fg(Color::White).bg(bg);
    set_str(buf, input_x, y0 + 1, display, is, input_w as u16);

    // Cursor
    let cursor_x = input_x + display.len().min(input_w) as u16;
    if cursor_x < x0 + bw - 1 {
        set_cell(buf, cursor_x, y0 + 1, "▏", Style::default().fg(t.help_key).bg(bg));
    }

    // Matched count
    let matched = state.flows.len();
    let info = format!("{} flows matched", matched);
    let info_s = Style::default().fg(Color::Indexed(240)).bg(bg);
    set_str(buf, x0 + 2, y0 + 2, &info, info_s, bw - 4);

    let hint = "Enter=apply  Esc=cancel  ^W=del word";
    set_str(buf, x0 + 2, y0 + 3, hint, info_s, bw - 4);
}

// ─── Pause overlay ───────────────────────────────────────────────────────────

fn draw_pause_overlay(frame: &mut Frame, area: Rect, _state: &AppState) {
    let bw = 40u16.min(area.width.saturating_sub(4));
    let bh = 7u16;
    let bg = Color::Indexed(236);
    let bs = Style::default().fg(Color::Indexed(196));
    let buf = frame.buffer_mut();
    let (x0, y0) = draw_box(buf, area, bw, bh, bg, bs);

    let ts = Style::default().fg(Color::Indexed(196)).bg(bg).add_modifier(Modifier::BOLD);
    let title = "⏸  PAUSED";
    let title_cw = title.chars().count() as u16;
    set_str(buf, x0 + (bw.saturating_sub(title_cw)) / 2, y0 + 2, title, ts, bw - 4);

    let info_s = Style::default().fg(Color::White).bg(bg);
    let info = "Data refresh is frozen";
    set_str(buf, x0 + (bw.saturating_sub(info.len() as u16)) / 2, y0 + 3, info, info_s, bw - 4);

    let hint_s = Style::default().fg(DIM_BORDER).bg(bg);
    let hint = "press P to resume";
    set_str(buf, x0 + (bw.saturating_sub(hint.len() as u16)) / 2, y0 + 5, hint, hint_s, bw - 4);
}

// ─── Status message ───────────────────────────────────────────────────────────

fn draw_status(frame: &mut Frame, area: Rect, state: &AppState, text: &str) {
    let t = &state.theme;
    let buf = frame.buffer_mut();
    let msg_len = text.chars().count() as u16 + 4;
    let x0 = (area.width.saturating_sub(msg_len)) / 2;
    // Position above header bar + totals (3 rows) + separator (1) + header (1) + border (1)
    let bottom_offset: u16 = 6 + if state.show_header { 1 } else { 0 };
    let y0 = area.height.saturating_sub(bottom_offset);
    let s = Style::default().fg(Color::Black).bg(t.help_key);
    set_str(buf, x0, y0, &format!(" {} ", text), s, msg_len);
}

// ─── Right-click tooltip ──────────────────────────────────────────────────────

// ─── Header hover tooltip ─────────────────────────────────────────────────────

/// Find which pipe-delimited segment the cursor x falls into.
fn segment_at_x(buf: &Buffer, hover_x: u16, hover_y: u16, bar_start_x: u16, bar_end_x: u16) -> Option<String> {
    // Read the rendered bar text from the buffer
    let mut bar_text = String::new();
    for x in bar_start_x..bar_end_x {
        let a = buf.area();
        if x < a.x + a.width && hover_y < a.y + a.height {
            bar_text.push_str(buf[(x, hover_y)].symbol());
        }
    }

    // Split by │ (U+2502) and find which segment the cursor falls into
    let rel_x = hover_x.saturating_sub(bar_start_x) as usize;
    let mut pos = 0usize;
    for segment in bar_text.split('│') {
        let seg_chars = segment.chars().count() + 1; // +1 for the pipe
        if rel_x < pos + seg_chars {
            let trimmed = segment.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
            return None;
        }
        pos += seg_chars;
    }
    bar_text.split('│').next_back().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

fn draw_header_hover_tooltip(frame: &mut Frame, area: Rect, state: &AppState) {
    let (hover_x, hover_y) = match state.hover.pos {
        Some(pos) => pos,
        None => return,
    };

    let margin: u16 = if state.show_border { 1 } else { 0 };
    let buf = frame.buffer_mut();

    let segment = match segment_at_x(buf, hover_x, hover_y, margin, area.width.saturating_sub(margin)) {
        Some(s) => s,
        None => return,
    };

    let lines = state.header_segment_tooltip(&segment);
    if lines.is_empty() { return; }

    // Render tooltip popup
    let t = &state.theme;
    let label_s = Style::default().fg(t.help_val).bg(t.help_bg);
    let val_s = Style::default().fg(t.help_key).bg(t.help_bg);
    let bs = Style::default().fg(t.help_border);
    let bg = t.help_bg;

    let max_label = lines.iter().map(|(l, _)| l.chars().count()).max().unwrap_or(0);
    let max_val = lines.iter().map(|(_, v)| v.chars().count()).max().unwrap_or(0);
    let inner_w = (max_label + 3 + max_val).max(20);
    let bw = (inner_w + 4) as u16;
    let bh = (lines.len() + 2) as u16;

    // Position above the header bar (since it's at the bottom)
    let x0 = if hover_x + bw + 2 < area.width { hover_x + 1 } else { area.width.saturating_sub(bw + 1) };
    let y0 = hover_y.saturating_sub(bh);

    // Fill + rounded border
    for y in y0..y0 + bh {
        for x in x0..x0 + bw {
            set_cell(buf, x, y, " ", Style::default().bg(bg));
        }
    }
    set_cell(buf, x0, y0, "╭", bs); set_cell(buf, x0 + bw - 1, y0, "╮", bs);
    set_cell(buf, x0, y0 + bh - 1, "╰", bs); set_cell(buf, x0 + bw - 1, y0 + bh - 1, "╯", bs);
    for x in x0 + 1..x0 + bw - 1 {
        set_cell(buf, x, y0, "─", bs); set_cell(buf, x, y0 + bh - 1, "─", bs);
    }
    for y in y0 + 1..y0 + bh - 1 {
        set_cell(buf, x0, y, "│", bs); set_cell(buf, x0 + bw - 1, y, "│", bs);
    }

    // Content
    for (i, (label, value)) in lines.iter().enumerate() {
        let ey = y0 + 1 + i as u16;
        if ey >= y0 + bh - 1 { break; }
        set_str(buf, x0 + 2, ey, label, label_s, max_label as u16 + 1);
        if !value.is_empty() {
            let vx = x0 + 2 + max_label as u16 + 2;
            let remaining = bw.saturating_sub(max_label as u16 + 5);
            set_str(buf, vx, ey, value, val_s, remaining);
        }
    }
}

// ─── Right-click tooltip ──────────────────────────────────────────────────────

fn draw_tooltip(frame: &mut Frame, area: Rect, state: &AppState) {
    let tt = &state.tooltip;
    let t = &state.theme;
    let buf = frame.buffer_mut();

    // Calculate box size from content
    let max_label = tt.lines.iter().map(|(l, _)| l.len()).max().unwrap_or(0);
    let max_val = tt.lines.iter().map(|(_, v)| v.len()).max().unwrap_or(0);
    let inner_w = (max_label + 3 + max_val).max(20);
    let bw = (inner_w + 4) as u16;
    let bh = (tt.lines.len() + 2) as u16;

    // Position near the click, but keep on screen
    let x0 = if tt.x + bw + 2 < area.width { tt.x + 1 } else { tt.x.saturating_sub(bw + 1) };
    let y0 = if tt.y + bh + 1 < area.height { tt.y } else { tt.y.saturating_sub(bh) };

    let bg = t.help_bg;
    let bs = Style::default().fg(t.help_border);
    let label_s = Style::default().fg(t.help_val).bg(bg);
    let val_s = Style::default().fg(t.help_key).bg(bg);

    // Fill + rounded border
    for y in y0..y0 + bh {
        for x in x0..x0 + bw {
            set_cell(buf, x, y, " ", Style::default().bg(bg));
        }
    }
    set_cell(buf, x0, y0, "╭", bs); set_cell(buf, x0 + bw - 1, y0, "╮", bs);
    set_cell(buf, x0, y0 + bh - 1, "╰", bs); set_cell(buf, x0 + bw - 1, y0 + bh - 1, "╯", bs);
    for x in x0 + 1..x0 + bw - 1 {
        set_cell(buf, x, y0, "─", bs); set_cell(buf, x, y0 + bh - 1, "─", bs);
    }
    for y in y0 + 1..y0 + bh - 1 {
        set_cell(buf, x0, y, "│", bs); set_cell(buf, x0 + bw - 1, y, "│", bs);
    }

    // Content
    for (i, (label, value)) in tt.lines.iter().enumerate() {
        let ey = y0 + 1 + i as u16;
        if ey >= y0 + bh - 1 { break; }
        if label.is_empty() && value.is_empty() {
            // Separator line
            for x in x0 + 1..x0 + bw - 1 {
                set_cell(buf, x, ey, "·", Style::default().fg(Color::Indexed(240)).bg(bg));
            }
        } else {
            set_str(buf, x0 + 2, ey, label, label_s, max_label as u16 + 1);
            set_str(buf, x0 + 2 + max_label as u16 + 2, ey, value, val_s, max_val as u16 + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── rate_to_frac ──

    #[test]
    fn rate_to_frac_zero() {
        assert_eq!(rate_to_frac(0.0), 0.0);
    }

    #[test]
    fn rate_to_frac_negative() {
        assert_eq!(rate_to_frac(-100.0), 0.0);
    }

    #[test]
    fn rate_to_frac_clamps_at_one() {
        // Huge rate should clamp to 1.0
        let f = rate_to_frac(1e30);
        assert!((f - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rate_to_frac_mid_value() {
        // 1 byte/s = 8 bits/s, log10(8) ~ 0.9, /9.0 ~ 0.1
        let f = rate_to_frac(1.0);
        assert!(f > 0.0 && f < 1.0);
    }

    #[test]
    fn rate_to_frac_monotonic() {
        let a = rate_to_frac(100.0);
        let b = rate_to_frac(1000.0);
        let c = rate_to_frac(10000.0);
        assert!(a < b);
        assert!(b < c);
    }

    // ── bar_length ──

    #[test]
    fn bar_length_zero_rate() {
        assert_eq!(bar_length(0.0, 80), 0);
    }

    #[test]
    fn bar_length_positive_rate() {
        let bl = bar_length(1_000_000.0, 100);
        assert!(bl > 0 && bl <= 100);
    }

    #[test]
    fn bar_length_zero_cols() {
        assert_eq!(bar_length(1000.0, 0), 0);
    }

    // ── draw_flows empty guard ──

    #[test]
    fn draw_flows_empty_flows_no_panic() {
        use crate::ui::app::AppState;
        use crate::util::resolver::Resolver;
        use crate::config::prefs::Prefs;
        use crate::ui::app::CliOverrides;

        let mut app = AppState::new(
            Resolver::new(false), true, true, false, true,
            &Prefs::default(), CliOverrides::default(),
        );
        // Simulate: scroll_offset > 0 but flows is empty
        app.scroll_offset = 10;
        assert!(app.flows.is_empty());

        // Render into a small terminal — should not panic
        let backend = ratatui::backend::TestBackend::new(80, 24);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal.draw(|frame| {
            draw(frame, &mut app);
        }).unwrap();
    }

    #[test]
    fn draw_flows_scroll_offset_beyond_flows_no_panic() {
        use crate::ui::app::AppState;
        use crate::util::resolver::Resolver;
        use crate::config::prefs::Prefs;
        use crate::ui::app::CliOverrides;
        use crate::data::tracker::FlowSnapshot;
        use crate::data::flow::{FlowKey, Protocol};

        let mut app = AppState::new(
            Resolver::new(false), true, true, false, true,
            &Prefs::default(), CliOverrides::default(),
        );
        app.flows = vec![FlowSnapshot {
            key: FlowKey {
                src: "10.0.0.1".parse().unwrap(),
                dst: "10.0.0.2".parse().unwrap(),
                src_port: 5000, dst_port: 80,
                protocol: Protocol::Tcp,
            },
            sent_2s: 100.0, sent_10s: 0.0, sent_40s: 0.0,
            recv_2s: 0.0, recv_10s: 0.0, recv_40s: 0.0,
            total_sent: 100, total_recv: 0,
            process_name: None, pid: None,
        }];
        app.scroll_offset = 100; // way beyond

        let backend = ratatui::backend::TestBackend::new(80, 24);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal.draw(|frame| {
            draw(frame, &mut app);
        }).unwrap();
    }
}
