use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::Frame;

use crate::config::theme::{Theme, ThemeName};
use crate::ui::app::{AppState, BarStyle};
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

    // Border support
    let border = state.show_border;
    let border_color = if state.paused { DIM_BORDER } else { state.theme.scale_line };
    let margin: u16 = if border { 1 } else { 0 };

    if border {
        let buf = frame.buffer_mut();
        let bs = Style::default().fg(border_color);
        let x1 = size.width.saturating_sub(1);
        let y1 = size.height.saturating_sub(1);
        // Corners first, then edges skip corner positions
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
        let tx = (size.width.saturating_sub(title.len() as u16)) / 2;
        let ts = if state.paused {
            Style::default().fg(Color::Indexed(196)).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(border_color).add_modifier(Modifier::BOLD)
        };
        set_str(buf, tx, 0, &title, ts, title.len() as u16);
    }

    // Inner area (inside borders)
    let inner = Rect {
        x: margin,
        y: margin,
        width: size.width.saturating_sub(margin * 2),
        height: size.height.saturating_sub(margin * 2),
    };

    let c = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Length(1), Constraint::Length(1), Constraint::Min(4),
        Constraint::Length(1), Constraint::Length(3),
    ]).split(inner);

    // Store flow area Y for mouse hit-testing
    state.flow_area_y = c[2].y;

    draw_scale_labels(frame, c[0], state);
    draw_scale_ticks(frame, c[1], state);
    draw_flows(frame, c[2], state);
    draw_separator(frame, c[3], state);
    draw_totals(frame, c[4], state);

    // Pause overlay
    if state.paused {
        draw_pause_overlay(frame, size, state);
    }

    // Overlays
    if state.theme_chooser.active { draw_theme_chooser(frame, size, state); }
    if state.filter_state.active { draw_filter_popup(frame, size, state); }
    if state.tooltip.active { draw_tooltip(frame, size, state); }
    if let Some(ref msg) = state.status_msg
        && !msg.expired() { draw_status(frame, size, state, &msg.text); }
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

fn draw_flows(frame: &mut Frame, area: Rect, state: &AppState) {
    if area.height < 1 || area.width < 30 { return; }
    let t = &state.theme;
    let w = area.width;
    let start = state.scroll_offset.min(state.flows.len().saturating_sub(1));
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

// ─── Bottom totals with bars ──────────────────────────────────────────────────

fn draw_separator(frame: &mut Frame, area: Rect, state: &AppState) {
    let buf = frame.buffer_mut();
    let s = Style::default().fg(state.theme.scale_line);
    for x in area.x..area.x + area.width { buf.set_string(x, area.y, "─", s); }
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
    x0: u16, bl: u16, bar_bg: Color, bar_fg: Color, bs: BarStyle)
{
    let mx = buf.area().x + buf.area().width;
    let my = buf.area().y + buf.area().height;
    for (i, ch) in text.chars().enumerate() {
        let cx = x + i as u16;
        if cx >= mx || y >= my { break; }
        let c = &mut buf[(cx, y)];
        c.set_char(ch);
        if cx < x0 + bl {
            // Inside bar region
            match bs {
                BarStyle::Solid => {
                    c.set_fg(bar_fg); c.set_bg(bar_bg);
                }
                BarStyle::Gradient => {
                    // Bright text on dark bar bg
                    c.set_fg(fg); c.set_bg(bar_bg);
                    c.set_style(c.style().add_modifier(Modifier::BOLD));
                }
                BarStyle::Thin => {
                    c.set_fg(fg); c.set_bg(bar_bg);
                    c.set_style(c.style().add_modifier(Modifier::BOLD | Modifier::UNDERLINED));
                }
                BarStyle::Ascii => {
                    c.set_fg(fg); c.set_bg(bar_bg);
                    c.set_style(c.style().add_modifier(Modifier::BOLD));
                }
            }
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
    if s.len() <= m { s.to_string() }
    else if m <= 1 { s.chars().next().map(|c| c.to_string()).unwrap_or_default() }
    else { let t: String = s.chars().take(m - 1).collect(); format!("{}~", t) }
}

// ─── Help modal (storageshower-style) ─────────────────────────────────────────

fn draw_help(frame: &mut Frame, area: Rect, state: &AppState) {
    let t = &state.theme;
    let buf = frame.buffer_mut();
    let bw = 90u16.min(area.width.saturating_sub(4));
    let bh = 30u16.min(area.height.saturating_sub(4));
    let bg = t.help_bg;
    let bs = Style::default().fg(t.help_border);
    let bgs = Style::default().fg(Color::White).bg(bg);
    let ks = Style::default().fg(t.help_key).bg(bg);
    let ts = Style::default().fg(t.help_title).bg(bg).add_modifier(Modifier::BOLD);
    let ss = Style::default().fg(t.help_section).bg(bg).add_modifier(Modifier::BOLD);

    let (x0, y0) = draw_box(buf, area, bw, bh, bg, bs);

    let ver = env!("CARGO_PKG_VERSION");
    let title = format!("⌨ IFTOPRS v{} — KEYBOARD SHORTCUTS", ver);
    set_str(buf, x0 + (bw.saturating_sub(title.len() as u16)) / 2, y0 + 1, &title, ts, bw - 2);
    let bl = "[ jacking into your packet stream ]";
    set_str(buf, x0 + (bw.saturating_sub(bl.len() as u16)) / 2, y0 + 2, bl, Style::default().fg(Color::Indexed(240)).bg(bg), bw - 2);

    let entries: [(&str, &[(&str, &str)]); 7] = [
        ("CAPTURE", &[("n","DNS toggle"),("N","Port names"),("p","Ports"),("Z","Processes"),("B","Bytes/bits"),("b","Bar style"),("T","Cumulative"),("P","Pause")]),
        ("SORT", &[("1","By 2s"),("2","By 10s"),("3","By 40s"),("<","By source"),(">","By dest"),("r","Reverse"),("o","Freeze order")]),
        ("NAV", &[("j/↓","Select next"),("k/↑","Select prev"),("^D","Half-page dn"),("^U","Half-page up"),("G/End","Jump last"),("Home","Jump first"),("Esc","Deselect")]),
        ("FILTER", &[("/","Search flows"),("0","Clear filter")]),
        ("ACTIONS", &[("e","Export flows"),("y","Copy selected"),("F","Pin/unpin ★")]),
        ("DISPLAY", &[("c","Theme chooser"),("t","Line mode"),("x","Toggle border"),("h/?","Toggle help"),("q","Quit")]),
        ("", &[]),
    ];

    let cw = ((bw as usize).saturating_sub(4)) / 3;
    let mut col = 0usize;
    let mut row = 0usize;
    for (section, keys) in &entries {
        if section.is_empty() { continue; }
        if row + keys.len() + 2 > (bh as usize - 6) { col += 1; row = 0; if col >= 3 { break; } }
        let cx = x0 + 2 + (col as u16) * cw as u16;
        let sy = y0 + 4 + row as u16;
        set_str(buf, cx, sy, section, ss, cw as u16);
        row += 1;
        for &(k, d) in *keys {
            let ey = y0 + 4 + row as u16;
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
    set_str(buf, x0 + (bw.saturating_sub(title.len() as u16)) / 2, y0 + 2, title, ts, bw - 4);

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
    let msg_len = text.len() as u16 + 4;
    let x0 = (area.width.saturating_sub(msg_len)) / 2;
    let y0 = area.height.saturating_sub(6);
    let s = Style::default().fg(Color::Black).bg(t.help_key);
    set_str(buf, x0, y0, &format!(" {} ", text), s, msg_len);
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
