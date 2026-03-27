use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::data::flow::{Direction, FlowKey, Protocol};

/// Result of parsing a single packet.
#[derive(Debug)]
pub struct ParsedPacket {
    pub key: FlowKey,
    pub direction: Direction,
    pub len: u64,
}

/// Parse an Ethernet frame, returning the flow key and byte count.
pub fn parse_ethernet(data: &[u8], local_net: Option<(IpAddr, u8)>) -> Option<ParsedPacket> {
    if data.len() < 14 {
        return None;
    }

    let ethertype = u16::from_be_bytes([data[12], data[13]]);
    let payload = &data[14..];

    match ethertype {
        0x0800 => parse_ipv4(payload, local_net),
        0x86DD => parse_ipv6(payload, local_net),
        // 802.1Q VLAN tag — skip 4 extra bytes
        0x8100 => {
            if data.len() < 18 {
                return None;
            }
            let inner_type = u16::from_be_bytes([data[16], data[17]]);
            let vlan_payload = &data[18..];
            match inner_type {
                0x0800 => parse_ipv4(vlan_payload, local_net),
                0x86DD => parse_ipv6(vlan_payload, local_net),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Parse a BSD loopback (DLT_NULL) frame.
pub fn parse_loopback(data: &[u8], local_net: Option<(IpAddr, u8)>) -> Option<ParsedPacket> {
    if data.len() < 4 {
        return None;
    }
    // 4-byte header: address family in host byte order
    let af = u32::from_ne_bytes([data[0], data[1], data[2], data[3]]);
    let payload = &data[4..];
    match af {
        2 => parse_ipv4(payload, local_net),   // AF_INET
        30 => parse_ipv6(payload, local_net),  // AF_INET6 on macOS
        10 => parse_ipv6(payload, local_net),  // AF_INET6 on Linux
        _ => None,
    }
}

/// Parse a Linux cooked capture (DLT_LINUX_SLL) frame.
pub fn parse_sll(data: &[u8], local_net: Option<(IpAddr, u8)>) -> Option<ParsedPacket> {
    if data.len() < 16 {
        return None;
    }
    let protocol = u16::from_be_bytes([data[14], data[15]]);
    let payload = &data[16..];
    match protocol {
        0x0800 => parse_ipv4(payload, local_net),
        0x86DD => parse_ipv6(payload, local_net),
        _ => None,
    }
}

/// Parse raw IP (DLT_RAW) — no link-layer header.
pub fn parse_raw(data: &[u8], local_net: Option<(IpAddr, u8)>) -> Option<ParsedPacket> {
    if data.is_empty() {
        return None;
    }
    let version = data[0] >> 4;
    match version {
        4 => parse_ipv4(data, local_net),
        6 => parse_ipv6(data, local_net),
        _ => None,
    }
}

fn parse_ipv4(data: &[u8], local_net: Option<(IpAddr, u8)>) -> Option<ParsedPacket> {
    if data.len() < 20 {
        return None;
    }

    let ihl = ((data[0] & 0x0F) as usize) * 4;
    let total_len = u16::from_be_bytes([data[2], data[3]]) as u64;
    let protocol_num = data[9];
    let src_ip = Ipv4Addr::new(data[12], data[13], data[14], data[15]);
    let dst_ip = Ipv4Addr::new(data[16], data[17], data[18], data[19]);

    let protocol = Protocol::from_ip_next_header(protocol_num);
    let (src_port, dst_port) = parse_ports(protocol_num, data, ihl);

    let src = IpAddr::V4(src_ip);
    let dst = IpAddr::V4(dst_ip);

    let direction = determine_direction(src, dst, local_net);

    let (key_src, key_dst, key_src_port, key_dst_port) = match direction {
        Direction::Sent => (src, dst, src_port, dst_port),
        Direction::Received => (dst, src, dst_port, src_port),
    };

    Some(ParsedPacket {
        key: FlowKey {
            src: key_src,
            dst: key_dst,
            src_port: key_src_port,
            dst_port: key_dst_port,
            protocol,
        },
        direction,
        len: total_len,
    })
}

fn parse_ipv6(data: &[u8], local_net: Option<(IpAddr, u8)>) -> Option<ParsedPacket> {
    if data.len() < 40 {
        return None;
    }

    let payload_len = u16::from_be_bytes([data[4], data[5]]) as u64;
    let total_len = payload_len + 40;
    let next_header = data[6];

    let src_ip = Ipv6Addr::from(<[u8; 16]>::try_from(&data[8..24]).ok()?);
    let dst_ip = Ipv6Addr::from(<[u8; 16]>::try_from(&data[24..40]).ok()?);

    let protocol = Protocol::from_ip_next_header(next_header);
    let (src_port, dst_port) = parse_ports(next_header, data, 40);

    let src = IpAddr::V6(src_ip);
    let dst = IpAddr::V6(dst_ip);

    let direction = determine_direction(src, dst, local_net);

    let (key_src, key_dst, key_src_port, key_dst_port) = match direction {
        Direction::Sent => (src, dst, src_port, dst_port),
        Direction::Received => (dst, src, dst_port, src_port),
    };

    Some(ParsedPacket {
        key: FlowKey {
            src: key_src,
            dst: key_dst,
            src_port: key_src_port,
            dst_port: key_dst_port,
            protocol,
        },
        direction,
        len: total_len,
    })
}

fn parse_ports(protocol: u8, data: &[u8], header_len: usize) -> (u16, u16) {
    match protocol {
        6 | 17 => {
            // TCP or UDP: first 4 bytes after IP header are src_port + dst_port
            if data.len() >= header_len + 4 {
                let src = u16::from_be_bytes([data[header_len], data[header_len + 1]]);
                let dst = u16::from_be_bytes([data[header_len + 2], data[header_len + 3]]);
                (src, dst)
            } else {
                (0, 0)
            }
        }
        _ => (0, 0),
    }
}

/// Determine packet direction based on local network.
/// If no local_net is configured, use src as "sent" (first-seen convention).
fn determine_direction(
    src: IpAddr,
    dst: IpAddr,
    local_net: Option<(IpAddr, u8)>,
) -> Direction {
    if let Some((net, prefix)) = local_net {
        let src_local = ip_in_network(src, net, prefix);
        let dst_local = ip_in_network(dst, net, prefix);
        if src_local && !dst_local {
            return Direction::Sent;
        }
        if !src_local && dst_local {
            return Direction::Received;
        }
    }
    // Both local or both remote — default to Sent
    Direction::Sent
}

pub(crate) fn ip_in_network(addr: IpAddr, network: IpAddr, prefix_len: u8) -> bool {
    match (addr, network) {
        (IpAddr::V4(a), IpAddr::V4(n)) => {
            if prefix_len >= 32 {
                return a == n;
            }
            let mask = !0u32 << (32 - prefix_len);
            (u32::from(a) & mask) == (u32::from(n) & mask)
        }
        (IpAddr::V6(a), IpAddr::V6(n)) => {
            if prefix_len >= 128 {
                return a == n;
            }
            let mask = !0u128 << (128 - prefix_len);
            (u128::from(a) & mask) == (u128::from(n) & mask)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ipv4_tcp_packet(src: [u8; 4], dst: [u8; 4], src_port: u16, dst_port: u16) -> Vec<u8> {
        let mut pkt = vec![0u8; 44]; // 14 eth + 20 ip + 8 tcp/udp + 2 pad
        // Ethernet header
        pkt[12] = 0x08; pkt[13] = 0x00; // IPv4
        // IPv4 header at offset 14
        pkt[14] = 0x45; // version 4, IHL 5
        pkt[16] = 0x00; pkt[17] = 30; // total_len = 30
        pkt[23] = 6; // TCP
        pkt[26..30].copy_from_slice(&src);
        pkt[30..34].copy_from_slice(&dst);
        // TCP ports at offset 34 (14 eth + 20 ip)
        pkt[34] = (src_port >> 8) as u8; pkt[35] = src_port as u8;
        pkt[36] = (dst_port >> 8) as u8; pkt[37] = dst_port as u8;
        pkt
    }

    #[test]
    fn parse_ethernet_ipv4_tcp() {
        let pkt = make_ipv4_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80);
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Tcp);
        assert_eq!(result.key.src_port, 12345);
        assert_eq!(result.key.dst_port, 80);
        assert_eq!(result.len, 30);
    }

    #[test]
    fn parse_ethernet_too_short() {
        assert!(parse_ethernet(&[0; 10], None).is_none());
    }

    #[test]
    fn parse_ethernet_unknown_ethertype() {
        let mut pkt = vec![0u8; 60];
        pkt[12] = 0xFF; pkt[13] = 0xFF;
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    #[test]
    fn parse_raw_ipv4() {
        let mut raw = vec![0u8; 30];
        raw[0] = 0x45; // version 4, IHL 5
        raw[2] = 0; raw[3] = 30;
        raw[9] = 17; // UDP
        raw[12..16].copy_from_slice(&[192, 168, 1, 1]);
        raw[16..20].copy_from_slice(&[8, 8, 8, 8]);
        raw[20] = 0x1F; raw[21] = 0x90; // src port 8080
        raw[22] = 0x00; raw[23] = 0x35; // dst port 53
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
        assert_eq!(result.key.src_port, 8080);
        assert_eq!(result.key.dst_port, 53);
    }

    #[test]
    fn parse_raw_empty() {
        assert!(parse_raw(&[], None).is_none());
    }

    #[test]
    fn parse_loopback_too_short() {
        assert!(parse_loopback(&[0, 0], None).is_none());
    }

    #[test]
    fn parse_sll_too_short() {
        assert!(parse_sll(&[0; 10], None).is_none());
    }

    #[test]
    fn ip_in_network_ipv4_match() {
        let addr: IpAddr = "192.168.1.100".parse().unwrap();
        let net: IpAddr = "192.168.1.0".parse().unwrap();
        assert!(ip_in_network(addr, net, 24));
    }

    #[test]
    fn ip_in_network_ipv4_no_match() {
        let addr: IpAddr = "192.168.2.100".parse().unwrap();
        let net: IpAddr = "192.168.1.0".parse().unwrap();
        assert!(!ip_in_network(addr, net, 24));
    }

    #[test]
    fn ip_in_network_ipv4_wide_mask() {
        let addr: IpAddr = "10.255.255.255".parse().unwrap();
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network(addr, net, 8));
    }

    #[test]
    fn ip_in_network_ipv4_host_mask() {
        let addr: IpAddr = "10.0.0.1".parse().unwrap();
        let net: IpAddr = "10.0.0.1".parse().unwrap();
        assert!(ip_in_network(addr, net, 32));
    }

    #[test]
    fn ip_in_network_mixed_families() {
        let v4: IpAddr = "10.0.0.1".parse().unwrap();
        let v6: IpAddr = "::1".parse().unwrap();
        assert!(!ip_in_network(v4, v6, 8));
    }

    #[test]
    fn direction_with_local_net() {
        let local: IpAddr = "192.168.1.0".parse().unwrap();
        let pkt = make_ipv4_tcp_packet([192, 168, 1, 5], [8, 8, 8, 8], 5000, 443);
        let result = parse_ethernet(&pkt, Some((local, 24))).unwrap();
        assert_eq!(result.direction, Direction::Sent);
    }

    #[test]
    fn direction_received() {
        let local: IpAddr = "192.168.1.0".parse().unwrap();
        let pkt = make_ipv4_tcp_packet([8, 8, 8, 8], [192, 168, 1, 5], 443, 5000);
        let result = parse_ethernet(&pkt, Some((local, 24))).unwrap();
        assert_eq!(result.direction, Direction::Received);
    }
}
