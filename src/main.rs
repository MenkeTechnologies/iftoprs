mod capture;
mod config;
mod data;
mod ui;
mod util;

use std::io;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
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

    // Process attribution thread — always running so Z toggle works at runtime
    let tracker_proc = tracker.clone();
    std::thread::Builder::new()
        .name("proc-lookup".into())
        .spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(500));
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
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    let mut app = AppState::new(
        resolver,
        !args.hide_ports,
        !args.no_bars,
        args.bytes,
        args.show_processes,
    );

    let result = run_app(&mut terminal, &mut app, &tracker, &mut rx);

    // Restore terminal
    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
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
            if let Event::Key(key) = event::read().context("Failed to read event")? {
                // Ctrl+C always quits
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c')
                {
                    return Ok(());
                }

                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('h') => app.show_help = !app.show_help,
                    KeyCode::Char('n') => {
                        app.resolver.toggle();
                        app.show_dns = app.resolver.is_enabled();
                    }
                    KeyCode::Char('N') => {
                        app.show_port_names = !app.show_port_names;
                    }
                    KeyCode::Char('p') => app.show_ports = !app.show_ports,
                    KeyCode::Char('b') => app.show_bars = !app.show_bars,
                    KeyCode::Char('B') => app.use_bytes = !app.use_bytes,
                    KeyCode::Char('t') => app.line_display = app.line_display.next(),
                    KeyCode::Char('T') => app.show_cumulative = !app.show_cumulative,
                    KeyCode::Char('Z') => app.show_processes = !app.show_processes,
                    KeyCode::Char('P') => app.paused = !app.paused,
                    KeyCode::Char('1') => {
                        app.sort_column = SortColumn::Avg2s;
                        app.frozen_order = false;
                    }
                    KeyCode::Char('2') => {
                        app.sort_column = SortColumn::Avg10s;
                        app.frozen_order = false;
                    }
                    KeyCode::Char('3') => {
                        app.sort_column = SortColumn::Avg40s;
                        app.frozen_order = false;
                    }
                    KeyCode::Char('<') => {
                        app.sort_column = SortColumn::SrcName;
                        app.frozen_order = false;
                    }
                    KeyCode::Char('>') => {
                        app.sort_column = SortColumn::DstName;
                        app.frozen_order = false;
                    }
                    KeyCode::Char('o') => app.frozen_order = !app.frozen_order,
                    KeyCode::Char('j') | KeyCode::Down => {
                        if app.scroll_offset < app.flows.len().saturating_sub(1) {
                            app.scroll_offset += 1;
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        app.scroll_offset = app.scroll_offset.saturating_sub(1);
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
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
