use std::net::IpAddr;

use anyhow::{Context, Result};
use pcap::{Capture, Device};
use tokio::sync::mpsc;

use crate::capture::parser::{self, ParsedPacket};

/// A packet event sent from the capture thread to the main loop.
pub struct PacketEvent {
    pub parsed: ParsedPacket,
}

/// Start capturing packets on the given interface (or default).
/// Returns a channel receiver that yields parsed packet events.
pub fn start_capture(
    interface: Option<String>,
    filter: Option<String>,
    promiscuous: bool,
    local_net: Option<(IpAddr, u8)>,
    tx: mpsc::UnboundedSender<PacketEvent>,
) -> Result<CaptureHandle> {
    let device = if let Some(ref name) = interface {
        Device::list()
            .context("Failed to list devices")?
            .into_iter()
            .find(|d| d.name == *name)
            .with_context(|| format!("Interface '{}' not found", name))?
    } else {
        Device::lookup()
            .context("Failed to lookup default device")?
            .context("No default device found")?
    };

    let mut cap = Capture::from_device(device)
        .context("Failed to open device")?
        .promisc(promiscuous)
        .snaplen(256)
        .timeout(100)
        .open()
        .context("Failed to activate capture")?;

    if let Some(ref f) = filter {
        cap.filter(f, true)
            .with_context(|| format!("Failed to set BPF filter: {}", f))?;
    }

    let datalink = cap.get_datalink();

    let handle = std::thread::Builder::new()
        .name("packet-capture".into())
        .spawn(move || {
            loop {
                match cap.next_packet() {
                    Ok(packet) => {
                        let parsed = match datalink {
                            pcap::Linktype::ETHERNET => {
                                parser::parse_ethernet(packet.data, local_net)
                            }
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
                            break; // receiver dropped
                        }
                    }
                    Err(pcap::Error::TimeoutExpired) => continue,
                    Err(_) => break,
                }
            }
        })
        .context("Failed to spawn capture thread")?;

    Ok(CaptureHandle { _thread: handle })
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
