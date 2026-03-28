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

    let key = FlowKey {
        src: key_src,
        dst: key_dst,
        src_port: key_src_port,
        dst_port: key_dst_port,
        protocol,
    };
    let (key, swapped) = key.normalize();
    let direction = if swapped {
        match direction {
            Direction::Sent => Direction::Received,
            Direction::Received => Direction::Sent,
        }
    } else {
        direction
    };

    Some(ParsedPacket {
        key,
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

    let key = FlowKey {
        src: key_src,
        dst: key_dst,
        src_port: key_src_port,
        dst_port: key_dst_port,
        protocol,
    };
    let (key, swapped) = key.normalize();
    let direction = if swapped {
        match direction {
            Direction::Sent => Direction::Received,
            Direction::Received => Direction::Sent,
        }
    } else {
        direction
    };

    Some(ParsedPacket {
        key,
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
        // Normalized: 8.8.8.8:53 < 192.168.1.1:8080
        assert_eq!(result.key.src, "8.8.8.8".parse::<IpAddr>().unwrap());
        assert_eq!(result.key.src_port, 53);
        assert_eq!(result.key.dst_port, 8080);
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
        // After normalization, 8.8.8.8:443 < 192.168.1.5:5000 so key is canonical
        assert_eq!(result.key.src, "8.8.8.8".parse::<IpAddr>().unwrap());
        assert_eq!(result.key.dst, "192.168.1.5".parse::<IpAddr>().unwrap());
        assert_eq!(result.direction, Direction::Received);
    }

    #[test]
    fn direction_received() {
        let local: IpAddr = "192.168.1.0".parse().unwrap();
        let pkt = make_ipv4_tcp_packet([8, 8, 8, 8], [192, 168, 1, 5], 443, 5000);
        let result = parse_ethernet(&pkt, Some((local, 24))).unwrap();
        // Same canonical key as direction_with_local_net
        assert_eq!(result.key.src, "8.8.8.8".parse::<IpAddr>().unwrap());
        assert_eq!(result.key.dst, "192.168.1.5".parse::<IpAddr>().unwrap());
        assert_eq!(result.direction, Direction::Sent);
    }

    // ── VLAN parsing ──

    fn make_vlan_ipv4_tcp_packet(src: [u8; 4], dst: [u8; 4], src_port: u16, dst_port: u16) -> Vec<u8> {
        let mut pkt = vec![0u8; 48]; // 14 eth + 4 vlan + 20 ip + 8 tcp + 2 pad
        // Ethernet header with VLAN tag
        pkt[12] = 0x81; pkt[13] = 0x00; // 802.1Q
        // VLAN tag (2 bytes TCI)
        pkt[14] = 0x00; pkt[15] = 0x64; // VLAN ID 100
        // Inner ethertype
        pkt[16] = 0x08; pkt[17] = 0x00; // IPv4
        // IPv4 header at offset 18
        pkt[18] = 0x45; // version 4, IHL 5
        pkt[20] = 0x00; pkt[21] = 30; // total_len = 30
        pkt[27] = 6; // TCP
        pkt[30..34].copy_from_slice(&src);
        pkt[34..38].copy_from_slice(&dst);
        // TCP ports at offset 38 (18 + 20)
        pkt[38] = (src_port >> 8) as u8; pkt[39] = src_port as u8;
        pkt[40] = (dst_port >> 8) as u8; pkt[41] = dst_port as u8;
        pkt
    }

    #[test]
    fn parse_vlan_ipv4_tcp() {
        let pkt = make_vlan_ipv4_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80);
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Tcp);
        assert_eq!(result.key.src_port, 12345);
        assert_eq!(result.key.dst_port, 80);
    }

    #[test]
    fn parse_vlan_too_short() {
        let mut pkt = vec![0u8; 16]; // short VLAN frame
        pkt[12] = 0x81; pkt[13] = 0x00;
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    #[test]
    fn parse_vlan_unknown_inner_ethertype() {
        let mut pkt = vec![0u8; 60];
        pkt[12] = 0x81; pkt[13] = 0x00; // 802.1Q
        pkt[16] = 0xFF; pkt[17] = 0xFF; // unknown inner
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    // ── IPv6 parsing ──

    fn make_ipv6_tcp_packet() -> Vec<u8> {
        let mut pkt = vec![0u8; 14 + 40 + 4]; // eth + ipv6 + ports
        // Ethernet header
        pkt[12] = 0x86; pkt[13] = 0xDD; // IPv6
        // IPv6 header at offset 14
        pkt[14] = 0x60; // version 6
        pkt[18] = 0x00; pkt[19] = 4; // payload length = 4
        pkt[20] = 6; // next header = TCP
        pkt[21] = 64; // hop limit
        // src = ::1
        pkt[37] = 1;
        // dst = ::2
        pkt[53] = 2;
        // TCP ports at offset 54
        pkt[54] = 0x1F; pkt[55] = 0x90; // src port 8080
        pkt[56] = 0x00; pkt[57] = 0x50; // dst port 80
        pkt
    }

    #[test]
    fn parse_ethernet_ipv6_tcp() {
        let pkt = make_ipv6_tcp_packet();
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Tcp);
        assert_eq!(result.key.src_port, 8080);
        assert_eq!(result.key.dst_port, 80);
        assert_eq!(result.len, 44); // 40 header + 4 payload
    }

    #[test]
    fn parse_ethernet_ipv6_too_short() {
        let mut pkt = vec![0u8; 30]; // eth + partial ipv6
        pkt[12] = 0x86; pkt[13] = 0xDD;
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    // ── Raw IPv6 ──

    #[test]
    fn parse_raw_ipv6() {
        let mut raw = vec![0u8; 44]; // ipv6 header + ports
        raw[0] = 0x60; // version 6
        raw[4] = 0x00; raw[5] = 4; // payload length
        raw[6] = 17; // UDP
        raw[7] = 64;
        raw[23] = 1; // src = ::1
        raw[39] = 2; // dst = ::2
        raw[40] = 0x00; raw[41] = 0x35; // src port 53
        raw[42] = 0x1F; raw[43] = 0x90; // dst port 8080
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
        assert_eq!(result.key.src_port, 53);
        assert_eq!(result.key.dst_port, 8080);
    }

    #[test]
    fn parse_raw_unknown_version() {
        let raw = vec![0x30; 20]; // version 3
        assert!(parse_raw(&raw, None).is_none());
    }

    // ── Loopback parsing ──

    #[test]
    fn parse_loopback_af_inet() {
        let mut pkt = vec![0u8; 24 + 4]; // 4 lb + 20 ip + 4 ports
        // AF_INET = 2 in host byte order
        let af_bytes = 2u32.to_ne_bytes();
        pkt[0..4].copy_from_slice(&af_bytes);
        // IPv4 header at offset 4
        pkt[4] = 0x45;
        pkt[6] = 0; pkt[7] = 24; // total_len
        pkt[13] = 6; // TCP
        pkt[16..20].copy_from_slice(&[127, 0, 0, 1]);
        pkt[20..24].copy_from_slice(&[127, 0, 0, 1]);
        pkt[24] = 0x1F; pkt[25] = 0x90; // src 8080
        pkt[26] = 0x00; pkt[27] = 0x50; // dst 80
        let result = parse_loopback(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Tcp);
        // Same IP, so normalized by port: 80 < 8080
        assert_eq!(result.key.src_port, 80);
        assert_eq!(result.key.dst_port, 8080);
    }

    #[test]
    fn parse_loopback_unknown_af() {
        let mut pkt = vec![0u8; 60];
        let af_bytes = 99u32.to_ne_bytes();
        pkt[0..4].copy_from_slice(&af_bytes);
        assert!(parse_loopback(&pkt, None).is_none());
    }

    // ── SLL parsing ──

    #[test]
    fn parse_sll_ipv4() {
        let mut pkt = vec![0u8; 16 + 24]; // 16 sll + 20 ip + 4 ports
        pkt[14] = 0x08; pkt[15] = 0x00; // IPv4
        // IPv4 header at offset 16
        pkt[16] = 0x45;
        pkt[18] = 0; pkt[19] = 24;
        pkt[25] = 17; // UDP
        pkt[28..32].copy_from_slice(&[10, 0, 0, 1]);
        pkt[32..36].copy_from_slice(&[10, 0, 0, 2]);
        pkt[36] = 0x00; pkt[37] = 0x35; // src 53
        pkt[38] = 0x1F; pkt[39] = 0x90; // dst 8080
        let result = parse_sll(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
        assert_eq!(result.key.src_port, 53);
    }

    #[test]
    fn parse_sll_unknown_protocol() {
        let mut pkt = vec![0u8; 60];
        pkt[14] = 0xFF; pkt[15] = 0xFF;
        assert!(parse_sll(&pkt, None).is_none());
    }

    // ── Direction edge cases ──

    #[test]
    fn direction_both_local_defaults_sent() {
        let local: IpAddr = "10.0.0.0".parse().unwrap();
        // Both src and dst in local net
        let pkt = make_ipv4_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 5000, 80);
        let result = parse_ethernet(&pkt, Some((local, 8))).unwrap();
        assert_eq!(result.direction, Direction::Sent);
    }

    #[test]
    fn direction_both_remote_defaults_sent() {
        let local: IpAddr = "192.168.1.0".parse().unwrap();
        // Both src and dst outside local net
        let pkt = make_ipv4_tcp_packet([8, 8, 8, 8], [1, 1, 1, 1], 443, 80);
        let result = parse_ethernet(&pkt, Some((local, 24))).unwrap();
        // Normalized: 1.1.1.1:80 < 8.8.8.8:443, so swapped, direction flips
        assert_eq!(result.key.src, "1.1.1.1".parse::<IpAddr>().unwrap());
        assert_eq!(result.direction, Direction::Received);
    }

    #[test]
    fn direction_no_local_net_defaults_sent() {
        let pkt = make_ipv4_tcp_packet([1, 2, 3, 4], [5, 6, 7, 8], 80, 443);
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.direction, Direction::Sent);
    }

    // ── ip_in_network IPv6 ──

    #[test]
    fn ip_in_network_ipv6_match() {
        let addr: IpAddr = "2001:db8::1".parse().unwrap();
        let net: IpAddr = "2001:db8::".parse().unwrap();
        assert!(ip_in_network(addr, net, 32));
    }

    #[test]
    fn ip_in_network_ipv6_no_match() {
        let addr: IpAddr = "2001:db9::1".parse().unwrap();
        let net: IpAddr = "2001:db8::".parse().unwrap();
        assert!(!ip_in_network(addr, net, 32));
    }

    #[test]
    fn ip_in_network_ipv6_host_mask() {
        let addr: IpAddr = "::1".parse().unwrap();
        let net: IpAddr = "::1".parse().unwrap();
        assert!(ip_in_network(addr, net, 128));
    }

    #[test]
    fn ip_in_network_ipv6_slash64() {
        let addr: IpAddr = "fe80::abcd:1234".parse().unwrap();
        let net: IpAddr = "fe80::".parse().unwrap();
        assert!(ip_in_network(addr, net, 64));
    }

    #[test]
    fn ip_in_network_ipv6_slash64_no_match() {
        let addr: IpAddr = "fe81::1".parse().unwrap();
        let net: IpAddr = "fe80::".parse().unwrap();
        assert!(!ip_in_network(addr, net, 64));
    }

    // ── Port parsing edge cases ──

    #[test]
    fn parse_icmp_has_no_ports() {
        let mut raw = vec![0u8; 28]; // ipv4 + 8 payload
        raw[0] = 0x45;
        raw[2] = 0; raw[3] = 28;
        raw[9] = 1; // ICMP
        raw[12..16].copy_from_slice(&[10, 0, 0, 1]);
        raw[16..20].copy_from_slice(&[10, 0, 0, 2]);
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Icmp);
        assert_eq!(result.key.src_port, 0);
        assert_eq!(result.key.dst_port, 0);
    }

    #[test]
    fn parse_truncated_tcp_ports() {
        // IPv4 header only, no room for ports
        let mut raw = vec![0u8; 20];
        raw[0] = 0x45;
        raw[2] = 0; raw[3] = 20;
        raw[9] = 6; // TCP
        raw[12..16].copy_from_slice(&[10, 0, 0, 1]);
        raw[16..20].copy_from_slice(&[10, 0, 0, 2]);
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.key.src_port, 0);
        assert_eq!(result.key.dst_port, 0);
    }

    #[test]
    fn parse_ipv4_udp_packet() {
        let mut pkt = vec![0u8; 44];
        pkt[12] = 0x08; pkt[13] = 0x00; // IPv4
        pkt[14] = 0x45;
        pkt[16] = 0x00; pkt[17] = 28;
        pkt[23] = 17; // UDP
        pkt[26..30].copy_from_slice(&[10, 0, 0, 1]);
        pkt[30..34].copy_from_slice(&[8, 8, 8, 8]);
        pkt[34] = 0xC0; pkt[35] = 0x00; // src port 49152
        pkt[36] = 0x00; pkt[37] = 0x35; // dst port 53
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
        // Normalized: 8.8.8.8:53 < 10.0.0.1:49152
        assert_eq!(result.key.src_port, 53);
        assert_eq!(result.key.dst_port, 49152);
    }

    // ── Normalized key: both directions produce same canonical key ──

    #[test]
    fn both_directions_same_canonical_key() {
        let local: IpAddr = "192.168.1.0".parse().unwrap();
        let sent_pkt = make_ipv4_tcp_packet([192, 168, 1, 5], [8, 8, 8, 8], 5000, 443);
        let recv_pkt = make_ipv4_tcp_packet([8, 8, 8, 8], [192, 168, 1, 5], 443, 5000);
        let sent = parse_ethernet(&sent_pkt, Some((local, 24))).unwrap();
        let recv = parse_ethernet(&recv_pkt, Some((local, 24))).unwrap();
        // Both produce the same canonical key
        assert_eq!(sent.key, recv.key);
        assert_eq!(sent.key.src, "8.8.8.8".parse::<IpAddr>().unwrap());
        assert_eq!(sent.key.dst, "192.168.1.5".parse::<IpAddr>().unwrap());
        assert_eq!(sent.key.src_port, 443);
        assert_eq!(sent.key.dst_port, 5000);
    }

    // ── VLAN with IPv6 ──

    #[test]
    fn parse_vlan_ipv6() {
        let mut pkt = vec![0u8; 18 + 40 + 4]; // eth+vlan + ipv6 + ports
        pkt[12] = 0x81; pkt[13] = 0x00; // 802.1Q
        pkt[16] = 0x86; pkt[17] = 0xDD; // IPv6 inner
        // IPv6 at offset 18
        pkt[18] = 0x60;
        pkt[22] = 0; pkt[23] = 4; // payload len
        pkt[24] = 17; // UDP
        pkt[41] = 1; // src ::1
        pkt[57] = 2; // dst ::2
        pkt[58] = 0x00; pkt[59] = 0x35; // src 53
        pkt[60] = 0x1F; pkt[61] = 0x90; // dst 8080
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
    }
}
