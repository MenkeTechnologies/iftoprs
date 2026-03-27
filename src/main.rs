mod capture;
mod config;
mod data;
mod ui;
mod util;

use std::io;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEvent, MouseEventKind, MouseButton, EnableMouseCapture, DisableMouseCapture};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;

use config::cli::Args;
use data::tracker::FlowTracker;
use ui::app::{AppState, SortColumn};
use util::resolver::Resolver;

fn main() -> Result<()> {
    let args = Args::parse();

    if args.help {
        config::cli::print_cyberpunk_help();
        return Ok(());
    }

    if args.version {
        println!("iftoprs {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if let Some(shell) = args.completions {
        Args::generate_completions(shell);
        return Ok(());
    }

    // List interfaces mode
    if args.list_interfaces {
        let interfaces = capture::sniffer::list_interfaces()?;
        println!("Available interfaces:");
        for iface in interfaces {
            println!("  {}", iface);
        }
        return Ok(());
    }

    let local_net = args.parse_net_filter().or_else(|| {
        // Auto-detect local IP from the capture interface
        auto_detect_local_net(args.interface.as_deref())
    });
    let resolver = Resolver::new(!args.no_dns);
    let tracker = FlowTracker::new();

    // Start packet capture
    let (tx, mut rx) = mpsc::unbounded_channel();
    let _capture_handle = capture::sniffer::start_capture(
        args.interface.clone(),
        args.filter.clone(),
        args.promiscuous,
        local_net,
        tx,
    )?;

    // Process attribution thread — refreshes the socket→pid table periodically,
    // then applies lookups to all flows that don't have process info yet.
    let tracker_proc = tracker.clone();
    std::thread::Builder::new()
        .name("proc-lookup".into())
        .spawn(move || {
            loop {
                // Refresh the entire socket→pid table (one lsof call for ALL sockets)
                util::procinfo::refresh_proc_table();
                std::thread::sleep(Duration::from_secs(2));

                // Apply lookups to flows
                let keys = tracker_proc.flow_keys();
                for key in keys {
                    if let Some((pid, name)) = util::lookup_process(
                        key.src,
                        key.src_port,
                        key.dst,
                        key.dst_port,
                        &key.protocol,
                    ) {
                        tracker_proc.set_process_info(&key, pid, name);
                    }
                }
            }
        })
        .context("Failed to spawn proc-lookup thread")?;

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    let prefs = config::prefs::load_prefs();
    let mut app = AppState::new(
        resolver,
        !args.hide_ports,
        !args.no_bars,
        args.bytes,
        !args.no_processes,
        &prefs,
    );

    let result = run_app(&mut terminal, &mut app, &tracker, &mut rx);

    // Restore terminal
    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), DisableMouseCapture, LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
    tracker: &FlowTracker,
    rx: &mut mpsc::UnboundedReceiver<capture::sniffer::PacketEvent>,
) -> Result<()> {
    let tick_rate = Duration::from_millis(33); // ~30 fps
    let mut last_tick = Instant::now();

    loop {
        // Drain packet events (non-blocking)
        while let Ok(event) = rx.try_recv() {
            tracker.record(
                event.parsed.key,
                event.parsed.direction,
                event.parsed.len,
            );
        }

        // Periodic rotation
        tracker.maybe_rotate();

        // Update display snapshot
        let (flows, totals) = tracker.snapshot();
        app.update_snapshot(flows, totals);

        // Render
        terminal.draw(|frame| {
            ui::render::draw(frame, app);
        })?;

        // Handle input
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout).context("Failed to poll events")? {
            let ev = event::read().context("Failed to read event")?;

            // Mouse events
            if let Event::Mouse(mouse) = ev {
                handle_mouse(app, mouse);
                continue;
            }

            // Keyboard events
            let Event::Key(key) = ev else { continue; };

                // Ctrl+C always quits
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c')
                {
                    return Ok(());
                }

                // Filter input mode
                if app.filter_state.active {
                    match key.code {
                        KeyCode::Enter => {
                            app.filter_state.active = false;
                            let f = app.filter_state.buf.clone();
                            app.screen_filter = if f.is_empty() { None } else { Some(f) };
                        }
                        KeyCode::Esc => {
                            app.filter_state.active = false;
                            app.screen_filter = app.filter_state.prev.clone();
                        }
                        KeyCode::Backspace => app.filter_state.backspace(),
                        KeyCode::Left => app.filter_state.left(),
                        KeyCode::Right => app.filter_state.right(),
                        KeyCode::Home => app.filter_state.home(),
                        KeyCode::End => app.filter_state.end(),
                        KeyCode::Char(ch) => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                match ch {
                                    'w' => app.filter_state.delete_word(),
                                    'a' => app.filter_state.home(),
                                    'e' => app.filter_state.end(),
                                    'k' => app.filter_state.kill_to_end(),
                                    'u' => { app.filter_state.buf.clear(); app.filter_state.cursor = 0; }
                                    _ => {}
                                }
                            } else {
                                app.filter_state.insert(ch);
                            }
                            // Live filter preview
                            let f = app.filter_state.buf.clone();
                            app.screen_filter = if f.is_empty() { None } else { Some(f) };
                        }
                        _ => {}
                    }
                    continue;
                }

                // Theme chooser mode
                if app.theme_chooser.active {
                    match key.code {
                        KeyCode::Char('j') | KeyCode::Down => {
                            let len = config::theme::ThemeName::ALL.len();
                            app.theme_chooser.selected = (app.theme_chooser.selected + 1) % len;
                            let name = config::theme::ThemeName::ALL[app.theme_chooser.selected];
                            app.set_theme(name);
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            let len = config::theme::ThemeName::ALL.len();
                            app.theme_chooser.selected = (app.theme_chooser.selected + len - 1) % len;
                            let name = config::theme::ThemeName::ALL[app.theme_chooser.selected];
                            app.set_theme(name);
                        }
                        KeyCode::Enter => {
                            app.theme_chooser.active = false;
                            app.save_prefs();
                        }
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('c') => {
                            app.theme_chooser.active = false;
                        }
                        _ => {}
                    }
                    continue;
                }

                // Ctrl+key combos
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('d') => app.page_down(),
                        KeyCode::Char('u') => app.page_up(),
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    // ── Quit ──
                    KeyCode::Char('q') => { app.save_prefs(); return Ok(()); }

                    // ── Help / overlays ──
                    KeyCode::Char('h') | KeyCode::Char('?') => app.show_help = !app.show_help,
                    KeyCode::Char('c') => {
                        app.show_help = false;
                        app.theme_chooser.open(app.theme_name);
                    }

                    // ── Filter ──
                    KeyCode::Char('/') => {
                        app.show_help = false;
                        app.filter_state.open(&app.screen_filter);
                    }
                    KeyCode::Char('0') => {
                        app.screen_filter = None;
                        app.set_status("Filter cleared");
                    }

                    // ── Actions ──
                    KeyCode::Char('e') => app.export(),
                    KeyCode::Char('y') => app.copy_selected(),
                    KeyCode::Char('F') => app.toggle_pin(),

                    // ── Display toggles ──
                    KeyCode::Char('n') => {
                        app.resolver.toggle();
                        app.show_dns = app.resolver.is_enabled();
                    }
                    KeyCode::Char('N') => app.show_port_names = !app.show_port_names,
                    KeyCode::Char('p') => app.show_ports = !app.show_ports,
                    KeyCode::Char('b') => {
                        app.bar_style = app.bar_style.next();
                        app.set_status(format!("Bar style: {}", app.bar_style.name()));
                    }
                    KeyCode::Char('B') => app.use_bytes = !app.use_bytes,
                    KeyCode::Char('t') => app.line_display = app.line_display.next(),
                    KeyCode::Char('T') => app.show_cumulative = !app.show_cumulative,
                    KeyCode::Char('Z') => app.show_processes = !app.show_processes,
                    KeyCode::Char('P') => app.paused = !app.paused,
                    KeyCode::Char('x') => {
                        app.show_border = !app.show_border;
                        app.set_status(if app.show_border { "Border: on" } else { "Border: off" });
                    }

                    // ── Sort ──
                    KeyCode::Char('1') => { app.sort_column = SortColumn::Avg2s; app.frozen_order = false; }
                    KeyCode::Char('2') => { app.sort_column = SortColumn::Avg10s; app.frozen_order = false; }
                    KeyCode::Char('3') => { app.sort_column = SortColumn::Avg40s; app.frozen_order = false; }
                    KeyCode::Char('<') => { app.sort_column = SortColumn::SrcName; app.frozen_order = false; }
                    KeyCode::Char('>') => { app.sort_column = SortColumn::DstName; app.frozen_order = false; }
                    KeyCode::Char('r') => {
                        app.sort_reverse = !app.sort_reverse;
                        app.set_status(if app.sort_reverse { "Sort: reversed" } else { "Sort: normal" });
                    }
                    KeyCode::Char('o') => app.frozen_order = !app.frozen_order,

                    // ── Navigation ──
                    KeyCode::Char('j') | KeyCode::Down => app.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => app.select_prev(),
                    KeyCode::Char('G') | KeyCode::End => app.jump_bottom(),
                    KeyCode::Home => app.jump_top(),
                    KeyCode::Esc => { app.selected = None; app.show_help = false; }

                    // ── Mouse scroll ──
                    _ => {}
                }
            }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

fn handle_mouse(app: &mut AppState, mouse: MouseEvent) {
    // Dismiss tooltip on any click
    if matches!(mouse.kind, MouseEventKind::Down(_)) {
        app.tooltip.active = false;
    }

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Click on flow row → select it
            let row = mouse.row;
            if row >= app.flow_area_y {
                let idx = app.scroll_offset + (row - app.flow_area_y) as usize;
                if idx < app.flows.len() {
                    app.selected = Some(idx);
                }
            }
        }
        MouseEventKind::Down(MouseButton::Right) => {
            // Right-click → show tooltip
            let row = mouse.row;
            if row >= app.flow_area_y {
                let idx = app.scroll_offset + (row - app.flow_area_y) as usize;
                if idx < app.flows.len() {
                    app.selected = Some(idx);
                    app.show_tooltip(idx, mouse.column, mouse.row);
                }
            }
        }
        MouseEventKind::ScrollDown => app.select_next(),
        MouseEventKind::ScrollUp => app.select_prev(),
        MouseEventKind::Down(MouseButton::Middle) => {
            // Middle-click → toggle pin
            let row = mouse.row;
            if row >= app.flow_area_y {
                let idx = app.scroll_offset + (row - app.flow_area_y) as usize;
                if idx < app.flows.len() {
                    app.selected = Some(idx);
                    app.toggle_pin();
                }
            }
        }
        _ => {}
    }
}

/// Auto-detect local network from the default/specified interface.
fn auto_detect_local_net(interface: Option<&str>) -> Option<(std::net::IpAddr, u8)> {
    let devices = pcap::Device::list().ok()?;
    let device = if let Some(name) = interface {
        devices.into_iter().find(|d| d.name == name)?
    } else {
        pcap::Device::lookup().ok()??
    };

    // Find the first IPv4 address on this interface
    for addr in &device.addresses {
        if let std::net::IpAddr::V4(ipv4) = addr.addr {
            if ipv4.is_loopback() {
                continue;
            }
            // Derive prefix from netmask if available
            let prefix = addr
                .netmask
                .and_then(|m| match m {
                    std::net::IpAddr::V4(mask) => {
                        Some(u32::from(mask).count_ones() as u8)
                    }
                    _ => None,
                })
                .unwrap_or(24); // default /24
            return Some((std::net::IpAddr::V4(ipv4), prefix));
        }
    }
    None
}
