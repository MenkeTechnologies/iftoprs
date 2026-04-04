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
        2 => parse_ipv4(payload, local_net),  // AF_INET
        30 => parse_ipv6(payload, local_net), // AF_INET6 on macOS
        10 => parse_ipv6(payload, local_net), // AF_INET6 on Linux
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
fn determine_direction(src: IpAddr, dst: IpAddr, local_net: Option<(IpAddr, u8)>) -> Direction {
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
    if prefix_len == 0 {
        return matches!(
            (addr, network),
            (IpAddr::V4(_), IpAddr::V4(_)) | (IpAddr::V6(_), IpAddr::V6(_))
        );
    }
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
        pkt[12] = 0x08;
        pkt[13] = 0x00; // IPv4
        // IPv4 header at offset 14
        pkt[14] = 0x45; // version 4, IHL 5
        pkt[16] = 0x00;
        pkt[17] = 30; // total_len = 30
        pkt[23] = 6; // TCP
        pkt[26..30].copy_from_slice(&src);
        pkt[30..34].copy_from_slice(&dst);
        // TCP ports at offset 34 (14 eth + 20 ip)
        pkt[34] = (src_port >> 8) as u8;
        pkt[35] = src_port as u8;
        pkt[36] = (dst_port >> 8) as u8;
        pkt[37] = dst_port as u8;
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
        pkt[12] = 0xFF;
        pkt[13] = 0xFF;
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    #[test]
    fn parse_ethernet_ipv4_payload_shorter_than_ipv4_header_returns_none() {
        // 14-byte Ethernet + 19-byte IPv4 payload (minimum IPv4 header is 20 bytes).
        let mut pkt = vec![0u8; 33];
        pkt[12] = 0x08;
        pkt[13] = 0x00;
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    #[test]
    fn parse_ethernet_ipv6_fixed_header_incomplete_returns_none() {
        // 14-byte Ethernet + 39-byte IPv6 payload (fixed header is 40 bytes).
        let mut pkt = vec![0u8; 53];
        pkt[12] = 0x86;
        pkt[13] = 0xDD;
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    #[test]
    fn parse_sll_ipv4_payload_shorter_than_ipv4_header_returns_none() {
        let mut pkt = vec![0u8; 35];
        pkt[14] = 0x08;
        pkt[15] = 0x00;
        assert!(parse_sll(&pkt, None).is_none());
    }

    #[test]
    fn parse_vlan_ipv4_inner_payload_truncated_returns_none() {
        // 14 (Ethernet) + 4 (802.1Q) + 19 (truncated IPv4).
        let mut pkt = vec![0u8; 37];
        pkt[12] = 0x81;
        pkt[13] = 0x00;
        pkt[16] = 0x08;
        pkt[17] = 0x00;
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    #[test]
    fn parse_raw_ipv6_payload_shorter_than_40_bytes_returns_none() {
        let mut raw = vec![0u8; 39];
        raw[0] = 0x60;
        assert!(parse_raw(&raw, None).is_none());
    }

    #[test]
    fn parse_raw_ipv4() {
        let mut raw = vec![0u8; 30];
        raw[0] = 0x45; // version 4, IHL 5
        raw[2] = 0;
        raw[3] = 30;
        raw[9] = 17; // UDP
        raw[12..16].copy_from_slice(&[192, 168, 1, 1]);
        raw[16..20].copy_from_slice(&[8, 8, 8, 8]);
        raw[20] = 0x1F;
        raw[21] = 0x90; // src port 8080
        raw[22] = 0x00;
        raw[23] = 0x35; // dst port 53
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
    fn parse_loopback_af_inet_without_ipv4_payload_returns_none() {
        let mut pkt = vec![0u8; 4];
        pkt[0..4].copy_from_slice(&2u32.to_ne_bytes()); // AF_INET, no IP body
        assert!(parse_loopback(&pkt, None).is_none());
    }

    #[test]
    fn parse_loopback_af_inet6_linux_without_ipv6_payload_returns_none() {
        let mut pkt = vec![0u8; 4];
        pkt[0..4].copy_from_slice(&10u32.to_ne_bytes()); // AF_INET6 on Linux, no IPv6 body
        assert!(parse_loopback(&pkt, None).is_none());
    }

    #[test]
    fn parse_loopback_af_inet6_macos_without_ipv6_payload_returns_none() {
        let mut pkt = vec![0u8; 4];
        pkt[0..4].copy_from_slice(&30u32.to_ne_bytes()); // AF_INET6 on macOS, no IPv6 body
        assert!(parse_loopback(&pkt, None).is_none());
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
    fn ip_in_network_mixed_families_prefix_zero_returns_false() {
        assert!(!ip_in_network(
            "192.0.2.1".parse().unwrap(),
            "::1".parse().unwrap(),
            0
        ));
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

    fn make_vlan_ipv4_tcp_packet(
        src: [u8; 4],
        dst: [u8; 4],
        src_port: u16,
        dst_port: u16,
    ) -> Vec<u8> {
        let mut pkt = vec![0u8; 48]; // 14 eth + 4 vlan + 20 ip + 8 tcp + 2 pad
        // Ethernet header with VLAN tag
        pkt[12] = 0x81;
        pkt[13] = 0x00; // 802.1Q
        // VLAN tag (2 bytes TCI)
        pkt[14] = 0x00;
        pkt[15] = 0x64; // VLAN ID 100
        // Inner ethertype
        pkt[16] = 0x08;
        pkt[17] = 0x00; // IPv4
        // IPv4 header at offset 18
        pkt[18] = 0x45; // version 4, IHL 5
        pkt[20] = 0x00;
        pkt[21] = 30; // total_len = 30
        pkt[27] = 6; // TCP
        pkt[30..34].copy_from_slice(&src);
        pkt[34..38].copy_from_slice(&dst);
        // TCP ports at offset 38 (18 + 20)
        pkt[38] = (src_port >> 8) as u8;
        pkt[39] = src_port as u8;
        pkt[40] = (dst_port >> 8) as u8;
        pkt[41] = dst_port as u8;
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
        pkt[12] = 0x81;
        pkt[13] = 0x00;
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    #[test]
    fn parse_vlan_unknown_inner_ethertype() {
        let mut pkt = vec![0u8; 60];
        pkt[12] = 0x81;
        pkt[13] = 0x00; // 802.1Q
        pkt[16] = 0xFF;
        pkt[17] = 0xFF; // unknown inner
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    // ── IPv6 parsing ──

    fn make_ipv6_tcp_packet() -> Vec<u8> {
        let mut pkt = vec![0u8; 14 + 40 + 4]; // eth + ipv6 + ports
        // Ethernet header
        pkt[12] = 0x86;
        pkt[13] = 0xDD; // IPv6
        // IPv6 header at offset 14
        pkt[14] = 0x60; // version 6
        pkt[18] = 0x00;
        pkt[19] = 4; // payload length = 4
        pkt[20] = 6; // next header = TCP
        pkt[21] = 64; // hop limit
        // src = ::1
        pkt[37] = 1;
        // dst = ::2
        pkt[53] = 2;
        // TCP ports at offset 54
        pkt[54] = 0x1F;
        pkt[55] = 0x90; // src port 8080
        pkt[56] = 0x00;
        pkt[57] = 0x50; // dst port 80
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
        pkt[12] = 0x86;
        pkt[13] = 0xDD;
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    // ── Raw IPv6 ──

    #[test]
    fn parse_raw_ipv6() {
        let mut raw = vec![0u8; 44]; // ipv6 header + ports
        raw[0] = 0x60; // version 6
        raw[4] = 0x00;
        raw[5] = 4; // payload length
        raw[6] = 17; // UDP
        raw[7] = 64;
        raw[23] = 1; // src = ::1
        raw[39] = 2; // dst = ::2
        raw[40] = 0x00;
        raw[41] = 0x35; // src port 53
        raw[42] = 0x1F;
        raw[43] = 0x90; // dst port 8080
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

    #[test]
    fn parse_raw_ip_version_zero_returns_none() {
        assert!(parse_raw(&[0u8], None).is_none());
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
        pkt[6] = 0;
        pkt[7] = 24; // total_len
        pkt[13] = 6; // TCP
        pkt[16..20].copy_from_slice(&[127, 0, 0, 1]);
        pkt[20..24].copy_from_slice(&[127, 0, 0, 1]);
        pkt[24] = 0x1F;
        pkt[25] = 0x90; // src 8080
        pkt[26] = 0x00;
        pkt[27] = 0x50; // dst 80
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

    #[test]
    fn parse_loopback_af_zero_returns_none() {
        let mut pkt = vec![0u8; 8];
        pkt[0..4].copy_from_slice(&0u32.to_ne_bytes());
        assert!(parse_loopback(&pkt, None).is_none());
    }

    // ── SLL parsing ──

    #[test]
    fn parse_sll_ipv4() {
        let mut pkt = vec![0u8; 16 + 24]; // 16 sll + 20 ip + 4 ports
        pkt[14] = 0x08;
        pkt[15] = 0x00; // IPv4
        // IPv4 header at offset 16
        pkt[16] = 0x45;
        pkt[18] = 0;
        pkt[19] = 24;
        pkt[25] = 17; // UDP
        pkt[28..32].copy_from_slice(&[10, 0, 0, 1]);
        pkt[32..36].copy_from_slice(&[10, 0, 0, 2]);
        pkt[36] = 0x00;
        pkt[37] = 0x35; // src 53
        pkt[38] = 0x1F;
        pkt[39] = 0x90; // dst 8080
        let result = parse_sll(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
        assert_eq!(result.key.src_port, 53);
    }

    #[test]
    fn parse_sll_unknown_protocol() {
        let mut pkt = vec![0u8; 60];
        pkt[14] = 0xFF;
        pkt[15] = 0xFF;
        assert!(parse_sll(&pkt, None).is_none());
    }

    #[test]
    fn parse_sll_ipv6_udp() {
        // SLL (16) + IPv6 (40) + first 4 bytes of UDP (ports)
        let mut pkt = vec![0u8; 16 + 40 + 4];
        pkt[14] = 0x86;
        pkt[15] = 0xDD;
        pkt[16] = 0x60;
        pkt[20] = 0x00;
        pkt[21] = 4; // payload length
        pkt[22] = 17; // UDP
        pkt[23] = 64;
        pkt[39] = 1; // src ::1 (last byte)
        pkt[55] = 2; // dst ::2 (last byte)
        pkt[56] = 0x00;
        pkt[57] = 0x35; // src 53
        pkt[58] = 0x1F;
        pkt[59] = 0x90; // dst 8080
        let result = parse_sll(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
        assert_eq!(result.key.src_port, 53);
        assert_eq!(result.key.dst_port, 8080);
        assert_eq!(result.len, 44);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn parse_loopback_af_inet6() {
        let mut pkt = vec![0u8; 4 + 44];
        pkt[0..4].copy_from_slice(&30u32.to_ne_bytes()); // AF_INET6 on macOS
        pkt[4] = 0x60;
        pkt[8] = 0x00;
        pkt[9] = 4;
        pkt[10] = 17; // UDP
        pkt[11] = 64;
        pkt[27] = 1; // src ::1
        pkt[43] = 2; // dst ::2
        pkt[44] = 0x00;
        pkt[45] = 0x35;
        pkt[46] = 0x1F;
        pkt[47] = 0x90;
        let result = parse_loopback(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
        assert_eq!(result.key.src_port, 53);
        assert_eq!(result.key.dst_port, 8080);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parse_loopback_af_inet6_linux() {
        let mut pkt = vec![0u8; 4 + 44];
        pkt[0..4].copy_from_slice(&10u32.to_ne_bytes()); // AF_INET6 on Linux
        pkt[4] = 0x60;
        pkt[8] = 0x00;
        pkt[9] = 4;
        pkt[10] = 17;
        pkt[11] = 64;
        pkt[27] = 1;
        pkt[43] = 2;
        pkt[44] = 0x00;
        pkt[45] = 0x35;
        pkt[46] = 0x1F;
        pkt[47] = 0x90;
        let result = parse_loopback(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
        assert_eq!(result.key.src_port, 53);
        assert_eq!(result.key.dst_port, 8080);
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
        raw[2] = 0;
        raw[3] = 28;
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
        raw[2] = 0;
        raw[3] = 20;
        raw[9] = 6; // TCP
        raw[12..16].copy_from_slice(&[10, 0, 0, 1]);
        raw[16..20].copy_from_slice(&[10, 0, 0, 2]);
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.key.src_port, 0);
        assert_eq!(result.key.dst_port, 0);
    }

    #[test]
    fn parse_raw_ipv4_tcp_ports_after_options() {
        // IHL = 6 → 24-byte IPv4 header; TCP ports start at offset 24, not 20.
        let mut raw = vec![0u8; 28];
        raw[0] = 0x46; // version 4, IHL 6
        raw[2] = 0;
        raw[3] = 28; // 24-byte IP header + 4 bytes (src/dst port only)
        raw[9] = 6; // TCP
        raw[12..16].copy_from_slice(&[10, 0, 0, 1]);
        raw[16..20].copy_from_slice(&[10, 0, 0, 2]);
        raw[20..24].copy_from_slice(&[0x01, 0x01, 0x01, 0x01]); // NOP padding
        raw[24] = 0x30;
        raw[25] = 0x39; // 12345
        raw[26] = 0x00;
        raw[27] = 0x50; // 80
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Tcp);
        assert_eq!(result.key.src, "10.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(result.key.dst, "10.0.0.2".parse::<IpAddr>().unwrap());
        assert_eq!(result.key.src_port, 12345);
        assert_eq!(result.key.dst_port, 80);
    }

    #[test]
    fn parse_ethernet_ipv4_tcp_ports_after_options() {
        // Same IHL=6 layout as `parse_raw_ipv4_tcp_ports_after_options`, with Ethernet prefix.
        let mut pkt = vec![0u8; 14 + 28];
        pkt[12] = 0x08;
        pkt[13] = 0x00;
        pkt[14] = 0x46; // version 4, IHL 6
        pkt[16] = 0x00;
        pkt[17] = 28;
        pkt[23] = 6; // TCP
        pkt[26..30].copy_from_slice(&[10, 0, 0, 1]);
        pkt[30..34].copy_from_slice(&[10, 0, 0, 2]);
        pkt[34..38].copy_from_slice(&[0x01, 0x01, 0x01, 0x01]);
        pkt[38] = 0x30;
        pkt[39] = 0x39;
        pkt[40] = 0x00;
        pkt[41] = 0x50;
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Tcp);
        assert_eq!(result.key.src, "10.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(result.key.dst, "10.0.0.2".parse::<IpAddr>().unwrap());
        assert_eq!(result.key.src_port, 12345);
        assert_eq!(result.key.dst_port, 80);
    }

    #[test]
    fn parse_ipv4_udp_packet() {
        let mut pkt = vec![0u8; 44];
        pkt[12] = 0x08;
        pkt[13] = 0x00; // IPv4
        pkt[14] = 0x45;
        pkt[16] = 0x00;
        pkt[17] = 28;
        pkt[23] = 17; // UDP
        pkt[26..30].copy_from_slice(&[10, 0, 0, 1]);
        pkt[30..34].copy_from_slice(&[8, 8, 8, 8]);
        pkt[34] = 0xC0;
        pkt[35] = 0x00; // src port 49152
        pkt[36] = 0x00;
        pkt[37] = 0x35; // dst port 53
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
        pkt[12] = 0x81;
        pkt[13] = 0x00; // 802.1Q
        pkt[16] = 0x86;
        pkt[17] = 0xDD; // IPv6 inner
        // IPv6 at offset 18
        pkt[18] = 0x60;
        pkt[22] = 0;
        pkt[23] = 4; // payload len
        pkt[24] = 17; // UDP
        pkt[41] = 1; // src ::1
        pkt[57] = 2; // dst ::2
        pkt[58] = 0x00;
        pkt[59] = 0x35; // src 53
        pkt[60] = 0x1F;
        pkt[61] = 0x90; // dst 8080
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
        assert_eq!(result.len, 44);
        assert_eq!(result.key.src_port, 53);
        assert_eq!(result.key.dst_port, 8080);
    }

    // ── Negative / truncation paths ──

    #[test]
    fn parse_raw_ipv4_too_short() {
        let mut raw = vec![0u8; 19];
        raw[0] = 0x45;
        assert!(parse_raw(&raw, None).is_none());
    }

    #[test]
    fn parse_raw_ipv6_too_short() {
        let mut raw = vec![0u8; 39];
        raw[0] = 0x60;
        assert!(parse_raw(&raw, None).is_none());
    }

    #[test]
    fn parse_ethernet_ipv4_payload_too_short_for_header() {
        let mut pkt = vec![0u8; 14 + 19];
        pkt[12] = 0x08;
        pkt[13] = 0x00;
        pkt[14] = 0x45;
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    #[test]
    fn parse_ethernet_arp_ethertype_returns_none() {
        let mut pkt = vec![0u8; 64];
        pkt[12] = 0x08;
        pkt[13] = 0x06; // ARP
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    #[test]
    fn parse_raw_ipv4_sctp_protocol() {
        let mut raw = vec![0u8; 28];
        raw[0] = 0x45;
        raw[2] = 0;
        raw[3] = 28;
        raw[9] = 132; // SCTP
        raw[12..16].copy_from_slice(&[10, 0, 0, 1]);
        raw[16..20].copy_from_slice(&[10, 0, 0, 2]);
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Other(132));
    }

    #[test]
    fn parse_raw_ipv6_icmpv6() {
        let mut raw = vec![0u8; 44];
        raw[0] = 0x60;
        raw[4] = 0x00;
        raw[5] = 4;
        raw[6] = 58; // ICMPv6
        raw[7] = 64;
        raw[23] = 1;
        raw[39] = 2;
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Icmp);
        assert_eq!(result.key.src_port, 0);
    }

    #[test]
    fn parse_raw_ipv6_routing_next_header_still_parses_addresses() {
        let mut raw = vec![0u8; 44];
        raw[0] = 0x60;
        raw[4] = 0x00;
        raw[5] = 4;
        raw[6] = 47; // GRE — not TCP/UDP; ports zero
        raw[7] = 64;
        raw[23] = 1;
        raw[39] = 2;
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Other(47));
        assert_eq!(result.key.src_port, 0);
        assert_eq!(result.key.dst_port, 0);
    }

    #[test]
    fn ip_in_network_ipv4_slash31() {
        let addr: IpAddr = "192.0.2.1".parse().unwrap();
        let net: IpAddr = "192.0.2.0".parse().unwrap();
        assert!(ip_in_network(addr, net, 31));
        let other: IpAddr = "192.0.2.3".parse().unwrap();
        assert!(!ip_in_network(other, net, 31));
    }

    #[test]
    fn ip_in_network_ipv4_slash1_entire_half_internet() {
        let addr: IpAddr = "127.0.0.1".parse().unwrap();
        let net: IpAddr = "0.0.0.0".parse().unwrap();
        assert!(ip_in_network(addr, net, 1));
    }

    #[test]
    fn parse_raw_ipv4_udp_ports_after_options() {
        let mut raw = vec![0u8; 28];
        raw[0] = 0x46;
        raw[2] = 0;
        raw[3] = 28;
        raw[9] = 17; // UDP
        raw[12..16].copy_from_slice(&[10, 0, 0, 1]);
        raw[16..20].copy_from_slice(&[10, 0, 0, 2]);
        raw[20..24].copy_from_slice(&[0x01, 0x01, 0x01, 0x01]);
        raw[24] = 0x12;
        raw[25] = 0x34;
        raw[26] = 0x56;
        raw[27] = 0x78;
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
        assert_eq!(result.key.src_port, 0x1234);
        assert_eq!(result.key.dst_port, 0x5678);
    }

    #[test]
    fn parse_ethernet_vlan_ipv4_tcp_ports_after_options() {
        let mut pkt = vec![0u8; 18 + 28];
        pkt[12] = 0x81;
        pkt[13] = 0x00;
        pkt[16] = 0x08;
        pkt[17] = 0x00;
        pkt[18] = 0x46;
        pkt[20] = 0x00;
        pkt[21] = 28;
        pkt[27] = 6;
        pkt[30..34].copy_from_slice(&[10, 0, 0, 1]);
        pkt[34..38].copy_from_slice(&[10, 0, 0, 2]);
        pkt[38..42].copy_from_slice(&[0x01, 0x01, 0x01, 0x01]);
        pkt[42] = 0x00;
        pkt[43] = 0x16;
        pkt[44] = 0x01;
        pkt[45] = 0xbb;
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Tcp);
        assert_eq!(result.key.src_port, 22);
        assert_eq!(result.key.dst_port, 443);
    }

    #[test]
    fn parse_sll_ipv4_tcp_ports_after_options() {
        let mut pkt = vec![0u8; 16 + 28];
        pkt[14] = 0x08;
        pkt[15] = 0x00;
        pkt[16] = 0x46;
        pkt[18] = 0x00;
        pkt[19] = 28;
        pkt[25] = 6;
        pkt[28..32].copy_from_slice(&[172, 16, 0, 1]);
        pkt[32..36].copy_from_slice(&[172, 16, 0, 2]);
        pkt[36..40].copy_from_slice(&[0x01, 0x01, 0x01, 0x01]);
        pkt[40] = 0x13;
        pkt[41] = 0xc0;
        pkt[42] = 0x14;
        pkt[43] = 0x50;
        let result = parse_sll(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Tcp);
        assert_eq!(result.key.src_port, 0x13c0);
        assert_eq!(result.key.dst_port, 0x1450);
    }

    #[test]
    fn direction_ipv6_udp_with_local_net() {
        let local: IpAddr = "2001:db8::".parse().unwrap();
        let mut raw = vec![0u8; 44];
        raw[0] = 0x60;
        raw[4] = 0x00;
        raw[5] = 4;
        raw[6] = 17;
        raw[7] = 64;
        // src 2001:db8::1
        raw[8] = 0x20;
        raw[9] = 0x01;
        raw[10] = 0x0d;
        raw[11] = 0xb8;
        raw[22] = 0x00;
        raw[23] = 0x01;
        // dst 2001:db9::1
        raw[24] = 0x20;
        raw[25] = 0x01;
        raw[26] = 0x0d;
        raw[27] = 0xb9;
        raw[38] = 0x00;
        raw[39] = 0x01;
        raw[40] = 0x00;
        raw[41] = 0x35;
        raw[42] = 0x00;
        raw[43] = 0x35;
        let result = parse_raw(&raw, Some((local, 32))).unwrap();
        assert_eq!(result.direction, Direction::Sent);
        assert_eq!(result.key.protocol, Protocol::Udp);
    }

    #[test]
    fn direction_ipv6_udp_received_from_internet() {
        let local: IpAddr = "2001:db8::".parse().unwrap();
        let mut raw = vec![0u8; 44];
        raw[0] = 0x60;
        raw[4] = 0x00;
        raw[5] = 4;
        raw[6] = 17;
        raw[7] = 64;
        // src 2001:db9::5
        raw[8] = 0x20;
        raw[9] = 0x01;
        raw[10] = 0x0d;
        raw[11] = 0xb9;
        raw[23] = 5;
        // dst 2001:db8::2
        raw[24] = 0x20;
        raw[25] = 0x01;
        raw[26] = 0x0d;
        raw[27] = 0xb8;
        raw[39] = 2;
        raw[40] = 0;
        raw[41] = 0x35;
        raw[42] = 0x01;
        raw[43] = 0xbb;
        let result = parse_raw(&raw, Some((local, 32))).unwrap();
        assert_eq!(result.direction, Direction::Received);
    }

    // ── Additional Ethernet / ICMP / VLAN / SLL coverage ──

    #[test]
    fn parse_ethernet_icmpv4() {
        let mut pkt = vec![0u8; 44];
        pkt[12] = 0x08;
        pkt[13] = 0x00;
        pkt[14] = 0x45;
        pkt[16] = 0x00;
        pkt[17] = 28;
        pkt[23] = 1; // ICMP
        pkt[26..30].copy_from_slice(&[192, 168, 0, 1]);
        pkt[30..34].copy_from_slice(&[192, 168, 0, 2]);
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Icmp);
        assert_eq!(result.len, 28);
    }

    #[test]
    fn parse_ethernet_ipv4_extreme_ihl_truncated_ports_zero() {
        // IHL = 15 → 60-byte header; buffer shorter — TCP ports fall back to 0
        let mut pkt = vec![0u8; 14 + 20];
        pkt[12] = 0x08;
        pkt[13] = 0x00;
        pkt[14] = 0x4f;
        pkt[16] = 0x00;
        pkt[17] = 20;
        pkt[23] = 6;
        pkt[26..30].copy_from_slice(&[10, 0, 0, 1]);
        pkt[30..34].copy_from_slice(&[10, 0, 0, 2]);
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.src_port, 0);
        assert_eq!(result.key.dst_port, 0);
    }

    #[test]
    fn parse_vlan_ipv4_udp_full() {
        let mut pkt = vec![0u8; 18 + 28];
        pkt[12] = 0x81;
        pkt[13] = 0x00;
        pkt[16] = 0x08;
        pkt[17] = 0x00;
        pkt[18] = 0x45;
        pkt[20] = 0x00;
        pkt[21] = 28;
        pkt[27] = 17;
        pkt[30..34].copy_from_slice(&[10, 10, 10, 1]);
        pkt[34..38].copy_from_slice(&[10, 10, 10, 2]);
        pkt[38] = 0x00;
        pkt[39] = 0x44;
        pkt[40] = 0x00;
        pkt[41] = 0x43;
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
        assert_eq!(result.len, 28);
    }

    #[test]
    fn parse_sll_icmpv4() {
        let mut pkt = vec![0u8; 16 + 28];
        pkt[14] = 0x08;
        pkt[15] = 0x00;
        pkt[16] = 0x45;
        pkt[18] = 0;
        pkt[19] = 28;
        pkt[25] = 1;
        pkt[28..32].copy_from_slice(&[8, 8, 4, 4]);
        pkt[32..36].copy_from_slice(&[1, 1, 1, 1]);
        let result = parse_sll(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Icmp);
    }

    #[test]
    fn parse_raw_ipv4_total_length_short_buffer_still_parses_header_fields() {
        // total_len claims 100 but buffer is 28 — parser uses declared total_len for `len` field
        let mut raw = vec![0u8; 28];
        raw[0] = 0x45;
        raw[2] = 0x00;
        raw[3] = 100;
        raw[9] = 6;
        raw[12..16].copy_from_slice(&[1, 1, 1, 1]);
        raw[16..20].copy_from_slice(&[2, 2, 2, 2]);
        raw[20] = 0x00;
        raw[21] = 0x16;
        raw[22] = 0x00;
        raw[23] = 0x50;
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.len, 100);
        assert_eq!(result.key.protocol, Protocol::Tcp);
    }

    #[test]
    fn ip_in_network_ipv6_prefix127() {
        let addr: IpAddr = "2001:db8::1".parse().unwrap();
        let net: IpAddr = "2001:db8::".parse().unwrap();
        assert!(ip_in_network(addr, net, 127));
    }

    #[test]
    fn ip_in_network_ipv6_distinct_at_bit_64() {
        let addr: IpAddr = "2001:db8:1::1".parse().unwrap();
        let net: IpAddr = "2001:db8::".parse().unwrap();
        assert!(!ip_in_network(addr, net, 64));
    }

    #[test]
    fn parse_loopback_ipv4_short_total_len() {
        let mut pkt = vec![0u8; 4 + 28];
        pkt[0..4].copy_from_slice(&2u32.to_ne_bytes());
        pkt[4] = 0x45;
        pkt[6] = 0;
        pkt[7] = 28;
        pkt[13] = 17;
        pkt[16..20].copy_from_slice(&[10, 0, 0, 1]);
        pkt[20..24].copy_from_slice(&[10, 0, 0, 2]);
        pkt[24] = 0x14;
        pkt[25] = 0xe9;
        pkt[26] = 0x14;
        pkt[27] = 0xe9;
        let result = parse_loopback(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
    }

    #[test]
    fn direction_ipv4_both_on_link_local_sent_default() {
        let local: IpAddr = "169.254.0.0".parse().unwrap();
        let pkt = make_ipv4_tcp_packet([169, 254, 0, 1], [169, 254, 0, 2], 1024, 1025);
        let result = parse_ethernet(&pkt, Some((local, 16))).unwrap();
        assert_eq!(result.direction, Direction::Sent);
    }

    #[test]
    fn parse_raw_invalid_ip_version_returns_none() {
        let raw = vec![0x50u8; 40]; // version 5
        assert!(parse_raw(&raw, None).is_none());
    }

    #[test]
    fn parse_ethernet_ipv6_udp_normalized_ports() {
        let mut pkt = vec![0u8; 14 + 44];
        pkt[12] = 0x86;
        pkt[13] = 0xdd;
        pkt[14] = 0x60;
        pkt[18] = 0x00;
        pkt[19] = 4; // payload length
        pkt[20] = 17; // UDP
        pkt[21] = 64;
        pkt[37] = 1; // src ::1
        pkt[53] = 2; // dst ::2
        pkt[54] = 0x1f;
        pkt[55] = 0x90; // src 8080
        pkt[56] = 0x00;
        pkt[57] = 0x50; // dst 80
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
        // ::1 < ::2 so key order preserves ::1 → ::2 with original ports
        assert_eq!(result.key.src_port, 8080);
        assert_eq!(result.key.dst_port, 80);
    }

    #[test]
    fn parse_sll_ipv6_tcp() {
        // SLL (16) + IPv6 (40) + TCP ports — TCP starts at byte 56
        let mut pkt = vec![0u8; 16 + 40 + 4];
        pkt[14] = 0x86;
        pkt[15] = 0xdd;
        pkt[16] = 0x60;
        pkt[18] = 0x00;
        pkt[19] = 4;
        pkt[22] = 6;
        pkt[23] = 64;
        pkt[39] = 1; // src ::1
        pkt[55] = 2; // dst ::2
        pkt[56] = 0x00;
        pkt[57] = 0x16; // src port 22
        pkt[58] = 0x00;
        pkt[59] = 0x50; // dst port 80
        let result = parse_sll(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Tcp);
        assert_eq!(result.key.src_port, 22);
        assert_eq!(result.key.dst_port, 80);
    }

    #[test]
    fn parse_raw_ipv4_max_ttl() {
        let mut raw = vec![0u8; 28];
        raw[0] = 0x45;
        raw[2] = 0;
        raw[3] = 28;
        raw[8] = 255; // TTL
        raw[9] = 6;
        raw[12..16].copy_from_slice(&[1, 1, 1, 1]);
        raw[16..20].copy_from_slice(&[2, 2, 2, 2]);
        raw[20] = 0;
        raw[21] = 0x16;
        raw[22] = 0;
        raw[23] = 0x50;
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Tcp);
    }

    #[test]
    fn ip_in_network_ipv4_slash30() {
        let net: IpAddr = "192.0.2.0".parse().unwrap();
        assert!(ip_in_network("192.0.2.1".parse().unwrap(), net, 30));
        assert!(!ip_in_network("192.0.2.4".parse().unwrap(), net, 30));
    }

    #[test]
    fn ip_in_network_ipv6_unique_local() {
        let addr: IpAddr = "fd00::1".parse().unwrap();
        let net: IpAddr = "fd00::".parse().unwrap();
        assert!(ip_in_network(addr, net, 8));
    }

    #[test]
    fn parse_vlan_ipv4_icmp_echo() {
        let mut pkt = vec![0u8; 18 + 28];
        pkt[12] = 0x81;
        pkt[13] = 0x00;
        pkt[16] = 0x08;
        pkt[17] = 0x00;
        pkt[18] = 0x45;
        pkt[20] = 0x00;
        pkt[21] = 28;
        pkt[27] = 1; // ICMP
        pkt[30..34].copy_from_slice(&[8, 8, 8, 8]);
        pkt[34..38].copy_from_slice(&[1, 1, 1, 1]);
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Icmp);
    }

    #[test]
    fn parse_raw_ipv4_udp_equal_ports_after_options() {
        let mut raw = vec![0u8; 28];
        raw[0] = 0x46;
        raw[2] = 0;
        raw[3] = 28;
        raw[9] = 17;
        raw[12..16].copy_from_slice(&[1, 1, 1, 1]);
        raw[16..20].copy_from_slice(&[2, 2, 2, 2]);
        raw[20..24].copy_from_slice(&[0x01, 0x01, 0x01, 0x01]);
        raw[24] = 0x00;
        raw[25] = 0x45; // src 69
        raw[26] = 0x00;
        raw[27] = 0x45; // dst 69
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
        assert_eq!(result.key.src_port, 69);
        assert_eq!(result.key.dst_port, 69);
    }

    #[test]
    fn parse_raw_ipv6_hop_limit_1() {
        let mut raw = vec![0u8; 44];
        raw[0] = 0x60;
        raw[4] = 0x00;
        raw[5] = 4;
        raw[6] = 17;
        raw[7] = 1;
        raw[23] = 1;
        raw[39] = 2;
        raw[40] = 0;
        raw[41] = 0x35;
        raw[42] = 0x01;
        raw[43] = 0xbb;
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
    }

    #[test]
    fn parse_ethernet_ipv4_tos_byte_nonzero() {
        let mut pkt = vec![0u8; 44];
        pkt[12] = 0x08;
        pkt[13] = 0x00;
        pkt[14] = 0x45;
        pkt[15] = 0x88; // DSCP / ECN
        pkt[16] = 0x00;
        pkt[17] = 30;
        pkt[23] = 6;
        pkt[26..30].copy_from_slice(&[10, 0, 0, 1]);
        pkt[30..34].copy_from_slice(&[10, 0, 0, 2]);
        pkt[34] = 0x00;
        pkt[35] = 0x16;
        pkt[36] = 0x00;
        pkt[37] = 0x50;
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Tcp);
    }

    #[test]
    fn parse_raw_ipv4_ttl_zero() {
        let mut raw = vec![0u8; 28];
        raw[0] = 0x45;
        raw[2] = 0;
        raw[3] = 28;
        raw[8] = 0;
        raw[9] = 17;
        raw[12..16].copy_from_slice(&[10, 0, 0, 1]);
        raw[16..20].copy_from_slice(&[10, 0, 0, 2]);
        raw[20] = 0x12;
        raw[21] = 0x34;
        raw[22] = 0x56;
        raw[23] = 0x78;
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
    }

    #[test]
    fn ip_in_network_ipv4_slash16() {
        let net: IpAddr = "172.16.0.0".parse().unwrap();
        assert!(ip_in_network("172.16.255.255".parse().unwrap(), net, 16));
        assert!(!ip_in_network("172.17.0.1".parse().unwrap(), net, 16));
    }

    #[test]
    fn direction_ipv4_private_to_public_canonical_key_and_direction() {
        let local: IpAddr = "10.0.0.0".parse().unwrap();
        let pkt = make_ipv4_tcp_packet([10, 0, 0, 50], [1, 1, 1, 1], 49152, 443);
        let result = parse_ethernet(&pkt, Some((local, 8))).unwrap();
        // Canonical order is 1.1.1.1 < 10.0.0.50 → key swaps vs packet; Sent flips to Received.
        assert_eq!(result.key.src, "1.1.1.1".parse::<IpAddr>().unwrap());
        assert_eq!(result.key.dst, "10.0.0.50".parse::<IpAddr>().unwrap());
        assert_eq!(result.direction, Direction::Received);
    }

    #[test]
    fn parse_ethernet_ipv4_fragment_id_field_present() {
        let mut pkt = vec![0u8; 44];
        pkt[12] = 0x08;
        pkt[13] = 0x00;
        pkt[14] = 0x45;
        pkt[18] = 0x12;
        pkt[19] = 0x34; // identification
        pkt[16] = 0x00;
        pkt[17] = 30;
        pkt[23] = 6;
        pkt[26..30].copy_from_slice(&[192, 168, 1, 1]);
        pkt[30..34].copy_from_slice(&[192, 168, 1, 2]);
        pkt[34] = 0x00;
        pkt[35] = 0x16;
        pkt[36] = 0x00;
        pkt[37] = 0x50;
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Tcp);
    }

    #[test]
    fn parse_vlan_ipv6_minimal_udp() {
        let mut pkt = vec![0u8; 18 + 44];
        pkt[12] = 0x81;
        pkt[13] = 0x00;
        pkt[16] = 0x86;
        pkt[17] = 0xdd;
        pkt[18] = 0x60;
        pkt[22] = 0;
        pkt[23] = 4;
        pkt[24] = 17;
        pkt[25] = 64;
        pkt[41] = 1;
        pkt[57] = 2;
        pkt[58] = 0;
        pkt[59] = 0x35;
        pkt[60] = 0x00;
        pkt[61] = 0x35;
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
    }

    #[test]
    fn ip_in_network_ipv4_slash8_boundary() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.255.255.255".parse().unwrap(), net, 8));
        assert!(!ip_in_network("11.0.0.1".parse().unwrap(), net, 8));
    }

    #[test]
    fn parse_raw_ipv4_minimum_datagram_tcp() {
        let mut raw = vec![0u8; 40];
        raw[0] = 0x45;
        raw[2] = 0;
        raw[3] = 40;
        raw[9] = 6;
        raw[12..16].copy_from_slice(&[0, 0, 0, 0]);
        raw[16..20].copy_from_slice(&[255, 255, 255, 255]);
        raw[20] = 0x00;
        raw[21] = 0x16;
        raw[22] = 0x00;
        raw[23] = 0x50;
        let result = parse_raw(&raw, None).unwrap();
        assert_eq!(result.len, 40);
        assert_eq!(result.key.protocol, Protocol::Tcp);
    }

    #[test]
    fn parse_sll_ipv4_udp_minimum() {
        let mut pkt = vec![0u8; 16 + 28];
        pkt[14] = 0x08;
        pkt[15] = 0x00;
        pkt[16] = 0x45;
        pkt[18] = 0;
        pkt[19] = 28;
        pkt[25] = 17;
        pkt[28..32].copy_from_slice(&[0, 0, 0, 0]);
        pkt[32..36].copy_from_slice(&[255, 255, 255, 255]);
        pkt[36] = 0x13;
        pkt[37] = 0x88;
        pkt[38] = 0x13;
        pkt[39] = 0x89;
        let result = parse_sll(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
    }

    #[test]
    fn parse_loopback_ipv4_protocol_udp_ports() {
        let mut pkt = vec![0u8; 4 + 28];
        pkt[0..4].copy_from_slice(&2u32.to_ne_bytes());
        pkt[4] = 0x45;
        pkt[6] = 0;
        pkt[7] = 28;
        pkt[13] = 17;
        pkt[16..20].copy_from_slice(&[0, 0, 0, 0]);
        pkt[20..24].copy_from_slice(&[255, 255, 255, 255]);
        pkt[24] = 0x14;
        pkt[25] = 0xe9;
        pkt[26] = 0x14;
        pkt[27] = 0xea;
        let result = parse_loopback(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Udp);
    }

    #[test]
    fn direction_ipv6_same_prefix_sent_when_src_local() {
        let local: IpAddr = "2001:db8::".parse().unwrap();
        let mut raw = vec![0u8; 44];
        raw[0] = 0x60;
        raw[4] = 0x00;
        raw[5] = 4;
        raw[6] = 17;
        raw[7] = 64;
        raw[8] = 0x20;
        raw[9] = 0x01;
        raw[10] = 0x0d;
        raw[11] = 0xb8;
        raw[23] = 1;
        raw[24] = 0x20;
        raw[25] = 0x01;
        raw[26] = 0x0d;
        raw[27] = 0xb9;
        raw[39] = 1;
        raw[40] = 0;
        raw[41] = 0x35;
        raw[42] = 0x00;
        raw[43] = 0x35;
        let result = parse_raw(&raw, Some((local, 32))).unwrap();
        assert_eq!(result.direction, Direction::Sent);
    }

    #[test]
    fn ip_in_network_ipv6_slash127_host_in_subnet() {
        let net: IpAddr = "2001:db8::".parse().unwrap();
        let host: IpAddr = "2001:db8::1".parse().unwrap();
        assert!(ip_in_network(host, net, 127));
    }

    #[test]
    fn ip_in_network_ipv4_slash31_point_to_point_pair() {
        let net: IpAddr = "192.0.2.0".parse().unwrap();
        assert!(ip_in_network("192.0.2.0".parse().unwrap(), net, 31));
        assert!(ip_in_network("192.0.2.1".parse().unwrap(), net, 31));
        assert!(!ip_in_network("192.0.2.2".parse().unwrap(), net, 31));
    }

    #[test]
    fn ip_in_network_ipv4_slash32_host_only() {
        let net: IpAddr = "198.51.100.5".parse().unwrap();
        assert!(ip_in_network("198.51.100.5".parse().unwrap(), net, 32));
        assert!(!ip_in_network("198.51.100.6".parse().unwrap(), net, 32));
    }

    #[test]
    fn ip_in_network_ipv6_slash32_documentation() {
        let net: IpAddr = "2001:db8::".parse().unwrap();
        assert!(ip_in_network("2001:db8::ffff".parse().unwrap(), net, 32));
        assert!(!ip_in_network("2001:db9::1".parse().unwrap(), net, 32));
    }

    #[test]
    fn ip_in_network_ipv4_slash24_typical_lan() {
        let net: IpAddr = "192.168.1.0".parse().unwrap();
        assert!(ip_in_network("192.168.1.100".parse().unwrap(), net, 24));
        assert!(!ip_in_network("192.168.2.1".parse().unwrap(), net, 24));
    }

    #[test]
    fn ip_in_network_ipv6_slash64_subnet() {
        let net: IpAddr = "2001:db8:1::".parse().unwrap();
        assert!(ip_in_network(
            "2001:db8:1::dead:beef".parse().unwrap(),
            net,
            64
        ));
        assert!(!ip_in_network("2001:db8:2::1".parse().unwrap(), net, 64));
    }

    #[test]
    fn ip_in_network_ipv6_slash48_same_site() {
        let net: IpAddr = "2001:db8::".parse().unwrap();
        // /48 fixes 2001:0db8:0000 — 2001:db8::1 is inside; 2001:db8:1::1 is not (third hextet ≠ 0).
        assert!(ip_in_network("2001:db8::1".parse().unwrap(), net, 48));
        assert!(!ip_in_network("2001:db8:1::1".parse().unwrap(), net, 48));
        assert!(!ip_in_network("2001:db9::1".parse().unwrap(), net, 48));
    }

    #[test]
    fn ip_in_network_ipv4_slash12_private_b_range() {
        let net: IpAddr = "172.16.0.0".parse().unwrap();
        assert!(ip_in_network("172.31.255.255".parse().unwrap(), net, 12));
        assert!(!ip_in_network("172.32.0.1".parse().unwrap(), net, 12));
    }

    #[test]
    fn parse_ethernet_ipv4_icmp_ports_zero() {
        let mut pkt = vec![0u8; 42];
        pkt[12] = 0x08;
        pkt[13] = 0x00;
        pkt[14] = 0x45;
        pkt[16] = 0x00;
        pkt[17] = 28;
        pkt[23] = 1;
        pkt[26..30].copy_from_slice(&[192, 0, 2, 1]);
        pkt[30..34].copy_from_slice(&[192, 0, 2, 2]);
        let result = parse_ethernet(&pkt, None).unwrap();
        assert_eq!(result.key.protocol, Protocol::Icmp);
        assert_eq!(result.key.src_port, 0);
        assert_eq!(result.key.dst_port, 0);
        assert_eq!(result.len, 28);
    }

    #[test]
    fn parse_raw_rejects_invalid_ip_version() {
        let mut raw = vec![0u8; 24];
        raw[0] = 0x55;
        assert!(parse_raw(&raw, None).is_none());
    }

    #[test]
    fn parse_ipv4_datagram_too_short_returns_none() {
        let mut pkt = vec![0u8; 33];
        pkt[12] = 0x08;
        pkt[13] = 0x00;
        pkt[14] = 0x45;
        pkt[16] = 0x00;
        pkt[17] = 19;
        pkt[23] = 6;
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    #[test]
    fn ip_in_network_ipv6_prefix_128_requires_exact_address() {
        let net: IpAddr = "2001:db8::1".parse().unwrap();
        assert!(ip_in_network("2001:db8::1".parse().unwrap(), net, 128));
        assert!(!ip_in_network("2001:db8::2".parse().unwrap(), net, 128));
    }

    #[test]
    fn parse_sll_unknown_ethertype_returns_none() {
        let mut pkt = vec![0u8; 20];
        pkt[14] = 0x08;
        pkt[15] = 0x06;
        assert!(parse_sll(&pkt, None).is_none());
    }

    #[test]
    fn parse_vlan_inner_too_short_returns_none() {
        let mut pkt = vec![0u8; 18];
        pkt[12] = 0x81;
        pkt[13] = 0x00;
        pkt[16] = 0x08;
        pkt[17] = 0x00;
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    #[test]
    fn ip_in_network_ipv6_slash0_all_addresses_match() {
        let net: IpAddr = "::".parse().unwrap();
        assert!(ip_in_network("::1".parse().unwrap(), net, 0));
        assert!(ip_in_network(
            "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff".parse().unwrap(),
            net,
            0
        ));
    }

    #[test]
    fn ip_in_network_ipv4_slash0_all_match() {
        let net: IpAddr = "0.0.0.0".parse().unwrap();
        assert!(ip_in_network("255.255.255.255".parse().unwrap(), net, 0));
    }

    #[test]
    fn parse_raw_ipv6_minimum_tcp_ports() {
        let mut raw = vec![0u8; 44];
        raw[0] = 0x60;
        raw[4] = 0x00;
        raw[5] = 4;
        raw[6] = 6;
        raw[7] = 64;
        raw[23] = 1;
        raw[39] = 2;
        raw[40] = 0x01;
        raw[41] = 0xbb;
        raw[42] = 0x00;
        raw[43] = 0x50;
        let r = parse_raw(&raw, None).unwrap();
        assert_eq!(r.key.protocol, Protocol::Tcp);
        assert_eq!(r.key.src_port, 443);
        assert_eq!(r.key.dst_port, 80);
    }

    #[test]
    fn parse_ethernet_too_short_for_vlan_tag() {
        let mut pkt = vec![0u8; 16];
        pkt[12] = 0x81;
        pkt[13] = 0x00;
        assert!(parse_ethernet(&pkt, None).is_none());
    }

    #[test]
    fn ip_in_network_ipv4_prefix_len_above_32_requires_exact_host_match() {
        let net: IpAddr = "198.51.100.10".parse().unwrap();
        assert!(ip_in_network("198.51.100.10".parse().unwrap(), net, 64));
        assert!(!ip_in_network("198.51.100.11".parse().unwrap(), net, 64));
    }

    #[test]
    fn ip_in_network_ipv6_prefix_len_above_128_requires_exact_host_match() {
        let net: IpAddr = "2001:db8::1".parse().unwrap();
        assert!(ip_in_network("2001:db8::1".parse().unwrap(), net, 200));
        assert!(!ip_in_network("2001:db8::2".parse().unwrap(), net, 200));
    }

    #[test]
    fn ip_in_network_ipv4_slash29_typical_lan_slice() {
        let net: IpAddr = "192.168.1.0".parse().unwrap();
        assert!(ip_in_network("192.168.1.3".parse().unwrap(), net, 29));
        assert!(!ip_in_network("192.168.1.8".parse().unwrap(), net, 29));
    }

    #[test]
    fn ip_in_network_ipv4_slash26_typical_isp_allocation() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.0.0.15".parse().unwrap(), net, 26));
        assert!(!ip_in_network("10.0.1.0".parse().unwrap(), net, 26));
    }

    #[test]
    fn ip_in_network_ipv6_slash56_link_local_subnet() {
        let net: IpAddr = "fe80::".parse().unwrap();
        assert!(ip_in_network("fe80::1".parse().unwrap(), net, 56));
        assert!(!ip_in_network("fe90::1".parse().unwrap(), net, 56));
    }

    #[test]
    fn ip_in_network_ipv4_slash28_sixteen_addresses() {
        let net: IpAddr = "198.51.100.0".parse().unwrap();
        assert!(ip_in_network("198.51.100.15".parse().unwrap(), net, 28));
        assert!(!ip_in_network("198.51.100.16".parse().unwrap(), net, 28));
    }

    #[test]
    fn ip_in_network_ipv4_slash17_half_class_b() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.0.127.255".parse().unwrap(), net, 17));
        assert!(!ip_in_network("10.0.128.0".parse().unwrap(), net, 17));
    }

    #[test]
    fn ip_in_network_ipv4_slash23_pair_of_slash24s() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.0.1.255".parse().unwrap(), net, 23));
        assert!(!ip_in_network("10.0.2.0".parse().unwrap(), net, 23));
    }

    #[test]
    fn ip_in_network_ipv4_slash19_private_class_b_slice() {
        let net: IpAddr = "172.16.0.0".parse().unwrap();
        assert!(ip_in_network("172.16.31.255".parse().unwrap(), net, 19));
        assert!(!ip_in_network("172.16.32.0".parse().unwrap(), net, 19));
    }

    #[test]
    fn ip_in_network_ipv4_slash20_sixteen_consecutive_class_c() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.0.15.255".parse().unwrap(), net, 20));
        assert!(!ip_in_network("10.0.16.0".parse().unwrap(), net, 20));
    }

    #[test]
    fn ip_in_network_ipv4_slash21_eight_consecutive_class_c() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.0.7.255".parse().unwrap(), net, 21));
        assert!(!ip_in_network("10.0.8.0".parse().unwrap(), net, 21));
    }

    #[test]
    fn ip_in_network_ipv4_slash22_four_consecutive_class_c() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.0.3.255".parse().unwrap(), net, 22));
        assert!(!ip_in_network("10.0.4.0".parse().unwrap(), net, 22));
    }

    #[test]
    fn ip_in_network_ipv4_slash18_private_class_a_slice() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.0.63.255".parse().unwrap(), net, 18));
        assert!(!ip_in_network("10.0.64.0".parse().unwrap(), net, 18));
    }

    #[test]
    fn ip_in_network_ipv4_slash25_half_class_c() {
        let net: IpAddr = "192.168.1.0".parse().unwrap();
        assert!(ip_in_network("192.168.1.127".parse().unwrap(), net, 25));
        assert!(!ip_in_network("192.168.1.128".parse().unwrap(), net, 25));
    }

    #[test]
    fn ip_in_network_ipv4_slash27_thirty_two_addresses() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.0.0.31".parse().unwrap(), net, 27));
        assert!(!ip_in_network("10.0.0.32".parse().unwrap(), net, 27));
    }

    #[test]
    fn ip_in_network_ipv4_slash15_two_class_b_blocks() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.1.255.255".parse().unwrap(), net, 15));
        assert!(!ip_in_network("10.2.0.0".parse().unwrap(), net, 15));
    }

    #[test]
    fn ip_in_network_ipv4_slash14_four_class_b_blocks() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.3.255.255".parse().unwrap(), net, 14));
        assert!(!ip_in_network("10.4.0.0".parse().unwrap(), net, 14));
    }

    #[test]
    fn ip_in_network_ipv4_slash13_eight_class_b_blocks() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.7.255.255".parse().unwrap(), net, 13));
        assert!(!ip_in_network("10.8.0.0".parse().unwrap(), net, 13));
    }

    #[test]
    fn ip_in_network_ipv4_slash12_sixteen_class_b_blocks() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.15.255.255".parse().unwrap(), net, 12));
        assert!(!ip_in_network("10.16.0.0".parse().unwrap(), net, 12));
    }

    #[test]
    fn ip_in_network_ipv4_slash11_thirty_two_class_b_blocks() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.31.255.255".parse().unwrap(), net, 11));
        assert!(!ip_in_network("10.32.0.0".parse().unwrap(), net, 11));
    }

    #[test]
    fn ip_in_network_ipv4_slash10_sixty_four_class_b_blocks() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.63.255.255".parse().unwrap(), net, 10));
        assert!(!ip_in_network("10.64.0.0".parse().unwrap(), net, 10));
    }

    #[test]
    fn ip_in_network_ipv4_slash9_half_class_a_ten_space() {
        let net: IpAddr = "10.0.0.0".parse().unwrap();
        assert!(ip_in_network("10.127.255.255".parse().unwrap(), net, 9));
        assert!(!ip_in_network("10.128.0.0".parse().unwrap(), net, 9));
    }

    #[test]
    fn ip_in_network_ipv6_slash96_documentation_prefix() {
        let net: IpAddr = "2001:db8::".parse().unwrap();
        assert!(ip_in_network(
            "2001:db8::ffff:ffff".parse().unwrap(),
            net,
            96
        ));
        assert!(!ip_in_network("2001:db8::1:0:0".parse().unwrap(), net, 96));
    }

    #[test]
    fn ip_in_network_ipv6_slash112_documentation_prefix() {
        let net: IpAddr = "2001:db8::".parse().unwrap();
        assert!(ip_in_network("2001:db8::ffff".parse().unwrap(), net, 112));
        assert!(!ip_in_network("2001:db8::1:0".parse().unwrap(), net, 112));
    }

    #[test]
    fn ip_in_network_ipv6_slash64_unique_local_subnet() {
        let net: IpAddr = "fd00::".parse().unwrap();
        assert!(ip_in_network(
            "fd00::ffff:ffff:ffff:ffff".parse().unwrap(),
            net,
            64
        ));
        assert!(!ip_in_network("fd00:0:0:1::1".parse().unwrap(), net, 64));
    }

    #[test]
    fn ip_in_network_ipv4_slash20_private_class_c_slice() {
        let net: IpAddr = "192.168.0.0".parse().unwrap();
        assert!(ip_in_network("192.168.15.255".parse().unwrap(), net, 20));
        assert!(!ip_in_network("192.168.16.0".parse().unwrap(), net, 20));
    }

    #[test]
    fn ip_in_network_ipv4_slash10_cgnat_shared_address_space() {
        let net: IpAddr = "100.64.0.0".parse().unwrap();
        assert!(ip_in_network("100.127.255.255".parse().unwrap(), net, 10));
        assert!(!ip_in_network("100.128.0.1".parse().unwrap(), net, 10));
    }

    #[test]
    fn ip_in_network_ipv4_slash24_documentation_test_net_3() {
        let net: IpAddr = "203.0.113.0".parse().unwrap();
        assert!(ip_in_network("203.0.113.255".parse().unwrap(), net, 24));
        assert!(!ip_in_network("203.0.114.1".parse().unwrap(), net, 24));
    }

    #[test]
    fn ip_in_network_ipv4_slash15_rfc2544_benchmark_space() {
        let net: IpAddr = "198.18.0.0".parse().unwrap();
        assert!(ip_in_network("198.19.255.255".parse().unwrap(), net, 15));
        assert!(!ip_in_network("198.20.0.1".parse().unwrap(), net, 15));
    }

    #[test]
    fn ip_in_network_ipv4_slash24_documentation_test_net_1() {
        let net: IpAddr = "192.0.2.0".parse().unwrap();
        assert!(ip_in_network("192.0.2.255".parse().unwrap(), net, 24));
        assert!(!ip_in_network("192.0.3.1".parse().unwrap(), net, 24));
    }

    #[test]
    fn ip_in_network_ipv4_slash24_documentation_test_net_2() {
        let net: IpAddr = "198.51.100.0".parse().unwrap();
        assert!(ip_in_network("198.51.100.255".parse().unwrap(), net, 24));
        assert!(!ip_in_network("198.51.101.1".parse().unwrap(), net, 24));
    }

    #[test]
    fn ip_in_network_ipv4_slash4_multicast() {
        let net: IpAddr = "224.0.0.0".parse().unwrap();
        assert!(ip_in_network("239.255.255.255".parse().unwrap(), net, 4));
        assert!(!ip_in_network("223.255.255.255".parse().unwrap(), net, 4));
    }

    #[test]
    fn ip_in_network_ipv6_slash8_multicast_ff00() {
        let net: IpAddr = "ff00::".parse().unwrap();
        assert!(ip_in_network("ff0e::1".parse().unwrap(), net, 8));
        assert!(!ip_in_network("fe80::1".parse().unwrap(), net, 8));
    }

    #[test]
    fn parse_loopback_af_inet_minimum_ipv4_icmp() {
        let mut pkt = vec![0u8; 24];
        pkt[0..4].copy_from_slice(&2u32.to_ne_bytes()); // AF_INET
        pkt[4] = 0x45; // IPv4, IHL 5
        pkt[6] = 0x00;
        pkt[7] = 20; // total length
        pkt[4 + 9] = 1; // ICMP
        pkt[4 + 12..4 + 16].copy_from_slice(&[10, 0, 0, 1]);
        pkt[4 + 16..4 + 20].copy_from_slice(&[10, 0, 0, 2]);
        let r = parse_loopback(&pkt, None).unwrap();
        assert_eq!(r.key.protocol, Protocol::Icmp);
        assert_eq!(r.key.src_port, 0);
        assert_eq!(r.key.dst_port, 0);
    }
}
