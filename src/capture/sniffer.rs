use std::net::IpAddr;
use std::time::Duration;

use anyhow::{Context, Result};
use pcap::{Capture, Device};
use tokio::sync::mpsc;

use crate::capture::parser::{self, ParsedPacket};

/// Maximum back-off between capture restart attempts.
const MAX_BACKOFF: Duration = Duration::from_secs(30);

/// A packet event sent from the capture thread to the main loop.
pub struct PacketEvent {
    pub parsed: ParsedPacket,
}

/// Start capturing packets on the given interface (or default).
/// Returns a channel receiver that yields parsed packet events.
///
/// The capture thread automatically restarts on transient pcap errors
/// (interface flaps, buffer issues, I/O errors) with exponential back-off
/// up to 30 seconds.  It only exits when the receiver is dropped (app
/// shutting down).
pub fn start_capture(
    interface: Option<String>,
    filter: Option<String>,
    promiscuous: bool,
    local_net: Option<(IpAddr, u8)>,
    tx: mpsc::UnboundedSender<PacketEvent>,
) -> Result<CaptureHandle> {
    // Validate the device once up-front so the caller gets an immediate error
    // for obviously wrong interface names.
    let _ = resolve_device(&interface)?;

    let handle = std::thread::Builder::new()
        .name("packet-capture".into())
        .spawn(move || {
            let mut backoff = Duration::from_millis(250);

            loop {
                // (Re-)open the capture device.
                let cap = match open_capture(&interface, &filter, promiscuous) {
                    Ok(c) => {
                        backoff = Duration::from_millis(250); // reset on success
                        c
                    }
                    Err(_) => {
                        if tx.is_closed() {
                            return;
                        }
                        std::thread::sleep(backoff);
                        backoff = (backoff * 2).min(MAX_BACKOFF);
                        continue;
                    }
                };

                let datalink = cap.datalink;

                match run_capture_loop(cap.handle, datalink, local_net, &tx) {
                    LoopExit::ReceiverDropped => return,
                    LoopExit::TransientError => {
                        if tx.is_closed() {
                            return;
                        }
                        std::thread::sleep(backoff);
                        backoff = (backoff * 2).min(MAX_BACKOFF);
                    }
                }
            }
        })
        .context("Failed to spawn capture thread")?;

    Ok(CaptureHandle { _thread: handle })
}

/// Why the inner capture loop exited.
enum LoopExit {
    /// The channel receiver was dropped — app is shutting down.
    ReceiverDropped,
    /// A transient pcap error occurred — worth retrying.
    TransientError,
}

struct OpenedCapture {
    handle: Capture<pcap::Active>,
    datalink: pcap::Linktype,
}

fn resolve_device(interface: &Option<String>) -> Result<Device> {
    if let Some(name) = interface {
        Device::list()
            .context("Failed to list devices")?
            .into_iter()
            .find(|d| d.name == *name)
            .with_context(|| format!("Interface '{}' not found", name))
    } else {
        Device::lookup()
            .context("Failed to lookup default device")?
            .context("No default device found")
    }
}

fn open_capture(
    interface: &Option<String>,
    filter: &Option<String>,
    promiscuous: bool,
) -> Result<OpenedCapture> {
    let device = resolve_device(interface)?;

    let mut cap = Capture::from_device(device)
        .context("Failed to open device")?
        .promisc(promiscuous)
        .snaplen(256)
        .timeout(100)
        .open()
        .context("Failed to activate capture")?;

    if let Some(f) = filter {
        cap.filter(f, true)
            .with_context(|| format!("Failed to set BPF filter: {}", f))?;
    }

    let datalink = cap.get_datalink();

    Ok(OpenedCapture {
        handle: cap,
        datalink,
    })
}

fn run_capture_loop(
    mut cap: Capture<pcap::Active>,
    datalink: pcap::Linktype,
    local_net: Option<(IpAddr, u8)>,
    tx: &mpsc::UnboundedSender<PacketEvent>,
) -> LoopExit {
    loop {
        match cap.next_packet() {
            Ok(packet) => {
                let parsed = match datalink {
                    pcap::Linktype::ETHERNET => parser::parse_ethernet(packet.data, local_net),
                    pcap::Linktype(0) => {
                        // DLT_NULL (BSD loopback)
                        parser::parse_loopback(packet.data, local_net)
                    }
                    pcap::Linktype(113) => {
                        // DLT_LINUX_SLL
                        parser::parse_sll(packet.data, local_net)
                    }
                    pcap::Linktype(101) => {
                        // DLT_RAW
                        parser::parse_raw(packet.data, local_net)
                    }
                    _ => {
                        // Try raw IP as fallback
                        parser::parse_raw(packet.data, local_net)
                    }
                };

                if let Some(p) = parsed
                    && tx.send(PacketEvent { parsed: p }).is_err()
                {
                    return LoopExit::ReceiverDropped;
                }
            }
            Err(pcap::Error::TimeoutExpired) => continue,
            Err(_) => return LoopExit::TransientError,
        }
    }
}

pub struct CaptureHandle {
    _thread: std::thread::JoinHandle<()>,
}

/// List available network interfaces.
pub fn list_interfaces() -> Result<Vec<String>> {
    Ok(Device::list()
        .context("Failed to list devices")?
        .into_iter()
        .map(|d| d.name)
        .collect())
}
