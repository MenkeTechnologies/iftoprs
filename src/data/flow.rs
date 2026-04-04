use std::fmt;
use std::net::IpAddr;

/// Uniquely identifies a network flow (bidirectional).
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct FlowKey {
    pub src: IpAddr,
    pub dst: IpAddr,
    pub src_port: u16,
    pub dst_port: u16,
    pub protocol: Protocol,
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp,
    Other(u8),
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
            Protocol::Icmp => write!(f, "ICMP"),
            Protocol::Other(n) => write!(f, "Proto({})", n),
        }
    }
}

impl Protocol {
    pub fn from_ip_next_header(val: u8) -> Self {
        match val {
            6 => Protocol::Tcp,
            17 => Protocol::Udp,
            1 | 58 => Protocol::Icmp,
            other => Protocol::Other(other),
        }
    }
}

/// Direction of a packet relative to the canonical flow key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Sent,     // src -> dst (matches key order)
    Received, // dst -> src (reversed)
}

impl FlowKey {
    /// Canonicalize the key so that (A:pA, B:pB) and (B:pB, A:pA) hash equally.
    /// Returns the normalized key and whether src/dst were swapped.
    pub fn normalize(self) -> (Self, bool) {
        // Zero-allocation comparison using u128 representation
        let src_ord = match self.src {
            IpAddr::V4(v4) => u128::from(u32::from(v4)),
            IpAddr::V6(v6) => u128::from(v6),
        };
        let dst_ord = match self.dst {
            IpAddr::V4(v4) => u128::from(u32::from(v4)),
            IpAddr::V6(v6) => u128::from(v6),
        };
        let swap = (src_ord, self.src_port) > (dst_ord, self.dst_port);
        if swap {
            (
                FlowKey {
                    src: self.dst,
                    dst: self.src,
                    src_port: self.dst_port,
                    dst_port: self.src_port,
                    protocol: self.protocol,
                },
                true,
            )
        } else {
            (self, false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_from_tcp() {
        assert_eq!(Protocol::from_ip_next_header(6), Protocol::Tcp);
    }

    #[test]
    fn protocol_from_udp() {
        assert_eq!(Protocol::from_ip_next_header(17), Protocol::Udp);
    }

    #[test]
    fn protocol_from_icmp_v4() {
        assert_eq!(Protocol::from_ip_next_header(1), Protocol::Icmp);
    }

    #[test]
    fn protocol_from_icmp_v6() {
        assert_eq!(Protocol::from_ip_next_header(58), Protocol::Icmp);
    }

    #[test]
    fn protocol_from_other() {
        assert_eq!(Protocol::from_ip_next_header(47), Protocol::Other(47));
    }

    #[test]
    fn protocol_from_sctp_is_other() {
        assert_eq!(Protocol::from_ip_next_header(132), Protocol::Other(132));
    }

    #[test]
    fn protocol_from_esp_is_other() {
        assert_eq!(Protocol::from_ip_next_header(50), Protocol::Other(50));
    }

    #[test]
    fn protocol_from_zero_is_other() {
        assert_eq!(Protocol::from_ip_next_header(0), Protocol::Other(0));
    }

    #[test]
    fn protocol_from_max_u8_is_other() {
        assert_eq!(Protocol::from_ip_next_header(255), Protocol::Other(255));
    }

    #[test]
    fn protocol_display() {
        assert_eq!(format!("{}", Protocol::Tcp), "TCP");
        assert_eq!(format!("{}", Protocol::Udp), "UDP");
        assert_eq!(format!("{}", Protocol::Icmp), "ICMP");
        assert_eq!(format!("{}", Protocol::Other(99)), "Proto(99)");
    }

    #[test]
    fn flow_key_equality() {
        let k1 = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 12345,
            dst_port: 80,
            protocol: Protocol::Tcp,
        };
        let k2 = k1;
        assert_eq!(k1, k2);
    }

    #[test]
    fn flow_key_inequality_port() {
        let k1 = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 80,
            dst_port: 80,
            protocol: Protocol::Tcp,
        };
        let k2 = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 81,
            dst_port: 80,
            protocol: Protocol::Tcp,
        };
        assert_ne!(k1, k2);
    }

    #[test]
    fn flow_key_inequality_protocol() {
        let k1 = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 80,
            dst_port: 80,
            protocol: Protocol::Tcp,
        };
        let k2 = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 80,
            dst_port: 80,
            protocol: Protocol::Udp,
        };
        assert_ne!(k1, k2);
    }

    #[test]
    fn flow_key_inequality_ip() {
        let k1 = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 80,
            dst_port: 80,
            protocol: Protocol::Tcp,
        };
        let k2 = FlowKey {
            src: "10.0.0.3".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 80,
            dst_port: 80,
            protocol: Protocol::Tcp,
        };
        assert_ne!(k1, k2);
    }

    #[test]
    fn flow_key_hash_consistency() {
        use std::collections::HashMap;
        let k1 = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 80,
            dst_port: 443,
            protocol: Protocol::Tcp,
        };
        let k2 = k1;
        let mut map = HashMap::new();
        map.insert(k1, 42);
        assert_eq!(map.get(&k2), Some(&42));
    }

    #[test]
    fn flow_key_ipv6() {
        let k = FlowKey {
            src: "::1".parse().unwrap(),
            dst: "::2".parse().unwrap(),
            src_port: 80,
            dst_port: 443,
            protocol: Protocol::Tcp,
        };
        let k2 = k;
        assert_eq!(k, k2);
    }

    #[test]
    fn direction_eq() {
        assert_eq!(Direction::Sent, Direction::Sent);
        assert_eq!(Direction::Received, Direction::Received);
        assert_ne!(Direction::Sent, Direction::Received);
    }

    #[test]
    fn protocol_debug() {
        let p = Protocol::Tcp;
        let s = format!("{:?}", p);
        assert_eq!(s, "Tcp");
    }

    #[test]
    fn protocol_other_display() {
        assert_eq!(format!("{}", Protocol::Other(0)), "Proto(0)");
        assert_eq!(format!("{}", Protocol::Other(255)), "Proto(255)");
    }

    #[test]
    fn protocol_copy() {
        let p = Protocol::Udp;
        let p2 = p;
        assert_eq!(p, p2);
    }

    #[test]
    fn normalize_swaps_when_src_greater() {
        let k = FlowKey {
            src: "10.0.0.2".parse().unwrap(),
            dst: "10.0.0.1".parse().unwrap(),
            src_port: 80,
            dst_port: 443,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(swapped);
        assert_eq!(n.src, "10.0.0.1".parse::<std::net::IpAddr>().unwrap());
        assert_eq!(n.dst, "10.0.0.2".parse::<std::net::IpAddr>().unwrap());
        assert_eq!(n.src_port, 443);
        assert_eq!(n.dst_port, 80);
    }

    #[test]
    fn normalize_no_swap_when_already_canonical() {
        let k = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 80,
            dst_port: 443,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(!swapped);
        assert_eq!(n.src, "10.0.0.1".parse::<std::net::IpAddr>().unwrap());
        assert_eq!(n.src_port, 80);
    }

    #[test]
    fn normalize_same_ip_sorts_by_port() {
        let k = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.1".parse().unwrap(),
            src_port: 8080,
            dst_port: 80,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(swapped);
        assert_eq!(n.src_port, 80);
        assert_eq!(n.dst_port, 8080);
    }

    #[test]
    fn normalize_reversed_pair_equals_original() {
        let k1 = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 5000,
            dst_port: 443,
            protocol: Protocol::Tcp,
        };
        let k2 = FlowKey {
            src: "10.0.0.2".parse().unwrap(),
            dst: "10.0.0.1".parse().unwrap(),
            src_port: 443,
            dst_port: 5000,
            protocol: Protocol::Tcp,
        };
        let (n1, _) = k1.normalize();
        let (n2, _) = k2.normalize();
        assert_eq!(n1, n2);
    }

    #[test]
    fn normalize_swaps_ipv6_when_address_order_requires() {
        let k = FlowKey {
            src: "2001:db8::2".parse().unwrap(),
            dst: "2001:db8::1".parse().unwrap(),
            src_port: 443,
            dst_port: 80,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(swapped);
        assert_eq!(n.src, "2001:db8::1".parse::<std::net::IpAddr>().unwrap());
        assert_eq!(n.dst, "2001:db8::2".parse::<std::net::IpAddr>().unwrap());
        assert_eq!(n.src_port, 80);
        assert_eq!(n.dst_port, 443);
    }

    #[test]
    fn normalize_ipv6_reversed_pair_equals_original() {
        let k1 = FlowKey {
            src: "2001:db8::1".parse().unwrap(),
            dst: "2001:db8::2".parse().unwrap(),
            src_port: 5000,
            dst_port: 443,
            protocol: Protocol::Udp,
        };
        let k2 = FlowKey {
            src: "2001:db8::2".parse().unwrap(),
            dst: "2001:db8::1".parse().unwrap(),
            src_port: 443,
            dst_port: 5000,
            protocol: Protocol::Udp,
        };
        let (n1, _) = k1.normalize();
        let (n2, _) = k2.normalize();
        assert_eq!(n1, n2);
    }

    #[test]
    fn normalize_identical_endpoints_same_port_no_swap() {
        let k = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.1".parse().unwrap(),
            src_port: 80,
            dst_port: 80,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(!swapped);
        assert_eq!(n.src_port, 80);
        assert_eq!(n.dst_port, 80);
    }

    #[test]
    fn normalize_same_ip_sorts_by_port_when_dst_port_lower() {
        let k = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.1".parse().unwrap(),
            src_port: 443,
            dst_port: 80,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(swapped);
        assert_eq!(n.src_port, 80);
        assert_eq!(n.dst_port, 443);
    }

    #[test]
    fn normalize_same_ipv6_sorts_by_port() {
        let k = FlowKey {
            src: "fe80::1".parse().unwrap(),
            dst: "fe80::1".parse().unwrap(),
            src_port: 5353,
            dst_port: 53,
            protocol: Protocol::Udp,
        };
        let (n, swapped) = k.normalize();
        assert!(swapped);
        assert_eq!(n.src_port, 53);
        assert_eq!(n.dst_port, 5353);
    }

    #[test]
    fn normalize_ipv4_port_tie_breaker_when_addrs_equal() {
        // Same IP; lower port should be canonical src_port after normalize.
        let k = FlowKey {
            src: "192.0.2.1".parse().unwrap(),
            dst: "192.0.2.1".parse().unwrap(),
            src_port: 1024,
            dst_port: 1025,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(!swapped);
        assert_eq!(n.src_port, 1024);
        assert_eq!(n.dst_port, 1025);
    }

    #[test]
    fn normalize_preserves_protocol_across_swap() {
        let k = FlowKey {
            src: "10.0.0.2".parse().unwrap(),
            dst: "10.0.0.1".parse().unwrap(),
            src_port: 22,
            dst_port: 22,
            protocol: Protocol::Other(132),
        };
        let (n, swapped) = k.normalize();
        assert!(swapped);
        assert_eq!(n.protocol, Protocol::Other(132));
    }

    #[test]
    fn normalize_double_swap_is_identity_for_key() {
        let k = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 100,
            dst_port: 200,
            protocol: Protocol::Udp,
        };
        let (n1, _) = k.normalize();
        let rev = FlowKey {
            src: k.dst,
            dst: k.src,
            src_port: k.dst_port,
            dst_port: k.src_port,
            protocol: k.protocol,
        };
        let (n2, _) = rev.normalize();
        assert_eq!(n1, n2);
    }

    #[test]
    fn normalize_ipv4_broadcast_vs_host_order() {
        let k = FlowKey {
            src: "255.255.255.255".parse().unwrap(),
            dst: "0.0.0.0".parse().unwrap(),
            src_port: 1,
            dst_port: 2,
            protocol: Protocol::Udp,
        };
        let (n, swapped) = k.normalize();
        // 0.0.0.0 < 255.255.255.255 as u32
        assert!(swapped);
        assert_eq!(n.src, "0.0.0.0".parse::<std::net::IpAddr>().unwrap());
    }

    #[test]
    fn normalize_ipv6_loopback_vs_multicast_order() {
        let k = FlowKey {
            src: "ff02::1".parse().unwrap(),
            dst: "::1".parse().unwrap(),
            src_port: 12345,
            dst_port: 80,
            protocol: Protocol::Udp,
        };
        let (n, swapped) = k.normalize();
        assert!(swapped);
        assert_eq!(n.src, "::1".parse::<std::net::IpAddr>().unwrap());
        assert_eq!(n.src_port, 80);
        assert_eq!(n.dst_port, 12345);
    }

    #[test]
    fn normalize_port_tie_same_addrs_no_swap() {
        let k = FlowKey {
            src: "203.0.113.5".parse().unwrap(),
            dst: "203.0.113.5".parse().unwrap(),
            src_port: 9999,
            dst_port: 9999,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(!swapped);
        assert_eq!(n.src_port, 9999);
    }

    #[test]
    fn protocol_from_icmp_matches_both_numbers() {
        assert_eq!(Protocol::from_ip_next_header(1), Protocol::Icmp);
        assert_eq!(Protocol::from_ip_next_header(58), Protocol::Icmp);
    }

    #[test]
    fn normalize_ipv6_documentation_prefix() {
        let k = FlowKey {
            src: "2001:db8::dead".parse().unwrap(),
            dst: "2001:db8::beef".parse().unwrap(),
            src_port: 53,
            dst_port: 5353,
            protocol: Protocol::Udp,
        };
        let (n, swapped) = k.normalize();
        // 0xbeef < 0xdead in the low bits → canonical address is beef first
        assert!(swapped);
        assert_eq!(n.src, "2001:db8::beef".parse::<std::net::IpAddr>().unwrap());
        assert_eq!(n.src_port, 5353);
    }

    #[test]
    fn normalize_ipv4_mapped_ipv6_orders_before_global_unicast() {
        let k = FlowKey {
            src: "2001:db8::1".parse().unwrap(),
            dst: "::ffff:192.0.2.1".parse().unwrap(),
            src_port: 100,
            dst_port: 200,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(swapped);
        assert_eq!(
            n.src,
            "::ffff:192.0.2.1".parse::<std::net::IpAddr>().unwrap()
        );
        assert_eq!(n.src_port, 200);
        assert_eq!(n.dst_port, 100);
    }

    #[test]
    fn flow_key_copy_leaves_original_unchanged() {
        let k = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 1,
            dst_port: 2,
            protocol: Protocol::Tcp,
        };
        let mut copy = k;
        copy.src_port = 99;
        assert_eq!(k.src_port, 1);
        assert_eq!(copy.src_port, 99);
    }

    #[test]
    fn protocol_other_max_display() {
        assert_eq!(format!("{}", Protocol::Other(255)), "Proto(255)");
    }

    #[test]
    fn protocol_eq_ignores_other_inner_when_same() {
        assert_eq!(Protocol::Other(7), Protocol::Other(7));
        assert_ne!(Protocol::Other(7), Protocol::Other(8));
    }

    #[test]
    fn normalize_ipv4_port_only_tiebreak() {
        let k = FlowKey {
            src: "10.0.0.2".parse().unwrap(),
            dst: "10.0.0.1".parse().unwrap(),
            src_port: 65535,
            dst_port: 1,
            protocol: Protocol::Udp,
        };
        let (n, swapped) = k.normalize();
        assert!(swapped);
        assert_eq!(n.src, "10.0.0.1".parse::<std::net::IpAddr>().unwrap());
        assert_eq!(n.src_port, 1);
        assert_eq!(n.dst_port, 65535);
    }

    #[test]
    fn direction_copy_eq() {
        let a = Direction::Sent;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn protocol_debug_other() {
        let s = format!("{:?}", Protocol::Other(99));
        assert!(s.contains("99") || s.contains("Other"));
    }

    #[test]
    fn flow_key_protocol_field_preserved_normalize() {
        let k = FlowKey {
            src: "10.0.0.2".parse().unwrap(),
            dst: "10.0.0.1".parse().unwrap(),
            src_port: 80,
            dst_port: 443,
            protocol: Protocol::Icmp,
        };
        let (n, _) = k.normalize();
        assert_eq!(n.protocol, Protocol::Icmp);
    }

    #[test]
    fn normalize_ipv6_same_address_ports_descending() {
        let k = FlowKey {
            src: "fe80::1".parse().unwrap(),
            dst: "fe80::1".parse().unwrap(),
            src_port: 40000,
            dst_port: 80,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(swapped);
        assert_eq!(n.src_port, 80);
        assert_eq!(n.dst_port, 40000);
    }

    #[test]
    fn direction_sent_received_distinct() {
        assert_ne!(Direction::Sent, Direction::Received);
    }

    #[test]
    fn normalize_mixed_ipv4_and_ipv6_orders_by_u128() {
        let v4 = "203.0.113.1".parse::<IpAddr>().unwrap();
        let v6 = "2001:db8::1".parse::<IpAddr>().unwrap();
        let k = FlowKey {
            src: v6,
            dst: v4,
            src_port: 443,
            dst_port: 80,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(swapped);
        assert_eq!(n.src, v4);
        assert_eq!(n.dst, v6);
        assert_eq!(n.src_port, 80);
        assert_eq!(n.dst_port, 443);
    }

    #[test]
    fn protocol_from_sctp_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(132), Protocol::Other(132));
    }

    #[test]
    fn protocol_from_igmp_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(2), Protocol::Other(2));
    }

    #[test]
    fn normalize_mixed_ipv4_ipv6_reverse_equals_canonical() {
        let v4 = "198.51.100.2".parse::<IpAddr>().unwrap();
        let v6 = "2001:db8::2".parse::<IpAddr>().unwrap();
        let a = FlowKey {
            src: v4,
            dst: v6,
            src_port: 1000,
            dst_port: 2000,
            protocol: Protocol::Udp,
        };
        let b = FlowKey {
            src: v6,
            dst: v4,
            src_port: 2000,
            dst_port: 1000,
            protocol: Protocol::Udp,
        };
        let (na, _) = a.normalize();
        let (nb, _) = b.normalize();
        assert_eq!(na, nb);
    }

    #[test]
    fn normalize_ipv4_same_address_equal_ports_identity() {
        let ip = "192.0.2.10".parse::<IpAddr>().unwrap();
        let k = FlowKey {
            src: ip,
            dst: ip,
            src_port: 443,
            dst_port: 443,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(!swapped);
        assert_eq!(n, k);
    }

    #[test]
    fn normalize_ipv6_same_address_equal_ports_identity() {
        let ip = "2001:db8::7".parse::<IpAddr>().unwrap();
        let k = FlowKey {
            src: ip,
            dst: ip,
            src_port: 53,
            dst_port: 53,
            protocol: Protocol::Udp,
        };
        let (n, swapped) = k.normalize();
        assert!(!swapped);
        assert_eq!(n, k);
    }

    #[test]
    fn normalize_ipv4_same_address_lower_port_becomes_src() {
        let ip = "10.0.0.1".parse::<IpAddr>().unwrap();
        let k = FlowKey {
            src: ip,
            dst: ip,
            src_port: 9000,
            dst_port: 80,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(swapped);
        assert_eq!(n.src_port, 80);
        assert_eq!(n.dst_port, 9000);
    }

    #[test]
    fn protocol_from_esp_next_header_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(50), Protocol::Other(50));
    }

    #[test]
    fn protocol_from_ah_next_header_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(51), Protocol::Other(51));
    }

    #[test]
    fn normalize_ipv4_equal_ports_orders_by_lower_address() {
        let k = FlowKey {
            src: "10.0.0.2".parse().unwrap(),
            dst: "10.0.0.1".parse().unwrap(),
            src_port: 443,
            dst_port: 443,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(swapped);
        assert_eq!(n.src, "10.0.0.1".parse::<IpAddr>().unwrap());
        assert_eq!(n.dst, "10.0.0.2".parse::<IpAddr>().unwrap());
    }

    #[test]
    fn protocol_from_ipv6_routing_header_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(43), Protocol::Other(43));
    }

    #[test]
    fn protocol_from_ip_in_ip_encapsulation_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(4), Protocol::Other(4));
    }

    #[test]
    fn protocol_from_shim6_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(140), Protocol::Other(140));
    }

    #[test]
    fn protocol_from_reserved_next_header_255_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(255), Protocol::Other(255));
    }

    #[test]
    fn protocol_other_wrapping_tcp_number_not_equal_to_tcp_variant() {
        assert_ne!(Protocol::Other(6), Protocol::Tcp);
    }

    #[test]
    fn protocol_from_ipv6_mobility_header_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(135), Protocol::Other(135));
    }

    #[test]
    fn normalize_ipv4_equal_ports_no_swap_when_src_address_lower() {
        let k = FlowKey {
            src: "10.0.0.1".parse().unwrap(),
            dst: "10.0.0.2".parse().unwrap(),
            src_port: 80,
            dst_port: 80,
            protocol: Protocol::Tcp,
        };
        let (n, swapped) = k.normalize();
        assert!(!swapped);
        assert_eq!(n, k);
    }

    #[test]
    fn protocol_from_l2tp_next_header_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(115), Protocol::Other(115));
    }

    #[test]
    fn protocol_from_ipv6_no_next_header_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(59), Protocol::Other(59));
    }

    #[test]
    fn protocol_from_dccp_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(33), Protocol::Other(33));
    }

    #[test]
    fn protocol_from_rsvp_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(46), Protocol::Other(46));
    }

    #[test]
    fn protocol_from_ipv6_encapsulation_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(41), Protocol::Other(41));
    }

    #[test]
    fn protocol_from_hip_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(139), Protocol::Other(139));
    }

    #[test]
    fn protocol_from_mpls_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(137), Protocol::Other(137));
    }

    #[test]
    fn protocol_from_udp_lite_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(136), Protocol::Other(136));
    }

    #[test]
    fn protocol_from_manet_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(138), Protocol::Other(138));
    }

    #[test]
    fn protocol_from_fibre_channel_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(133), Protocol::Other(133));
    }

    #[test]
    fn protocol_from_rohc_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(142), Protocol::Other(142));
    }

    #[test]
    fn protocol_from_ethernet_in_ip_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(143), Protocol::Other(143));
    }

    #[test]
    fn protocol_from_ip_proto_144_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(144), Protocol::Other(144));
    }

    #[test]
    fn protocol_from_ip_proto_145_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(145), Protocol::Other(145));
    }

    #[test]
    fn protocol_from_ip_proto_146_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(146), Protocol::Other(146));
    }

    #[test]
    fn protocol_from_ip_proto_147_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(147), Protocol::Other(147));
    }

    #[test]
    fn protocol_from_ip_proto_148_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(148), Protocol::Other(148));
    }

    #[test]
    fn protocol_from_ip_proto_149_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(149), Protocol::Other(149));
    }

    #[test]
    fn protocol_from_ip_proto_150_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(150), Protocol::Other(150));
    }

    #[test]
    fn protocol_from_ip_proto_151_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(151), Protocol::Other(151));
    }

    #[test]
    fn protocol_from_ip_proto_152_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(152), Protocol::Other(152));
    }

    #[test]
    fn protocol_from_ip_proto_153_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(153), Protocol::Other(153));
    }

    #[test]
    fn protocol_from_ip_proto_154_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(154), Protocol::Other(154));
    }

    #[test]
    fn protocol_from_ip_proto_155_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(155), Protocol::Other(155));
    }

    #[test]
    fn protocol_from_ip_proto_156_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(156), Protocol::Other(156));
    }

    #[test]
    fn protocol_from_ip_proto_157_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(157), Protocol::Other(157));
    }

    #[test]
    fn protocol_from_ip_proto_158_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(158), Protocol::Other(158));
    }

    #[test]
    fn protocol_from_ip_proto_159_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(159), Protocol::Other(159));
    }

    #[test]
    fn protocol_from_ip_proto_160_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(160), Protocol::Other(160));
    }

    #[test]
    fn protocol_from_ip_proto_161_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(161), Protocol::Other(161));
    }

    #[test]
    fn protocol_from_ip_proto_162_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(162), Protocol::Other(162));
    }

    #[test]
    fn protocol_from_ip_proto_163_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(163), Protocol::Other(163));
    }

    #[test]
    fn protocol_from_ip_proto_164_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(164), Protocol::Other(164));
    }

    #[test]
    fn protocol_from_ip_proto_165_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(165), Protocol::Other(165));
    }

    #[test]
    fn protocol_from_ip_proto_166_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(166), Protocol::Other(166));
    }

    #[test]
    fn protocol_from_ip_proto_167_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(167), Protocol::Other(167));
    }

    #[test]
    fn protocol_from_ip_proto_168_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(168), Protocol::Other(168));
    }

    #[test]
    fn protocol_from_ip_proto_169_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(169), Protocol::Other(169));
    }

    #[test]
    fn protocol_from_ip_proto_170_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(170), Protocol::Other(170));
    }

    #[test]
    fn protocol_from_ip_proto_171_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(171), Protocol::Other(171));
    }

    #[test]
    fn protocol_from_ip_proto_172_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(172), Protocol::Other(172));
    }

    #[test]
    fn protocol_from_ip_proto_173_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(173), Protocol::Other(173));
    }

    #[test]
    fn protocol_from_ip_proto_174_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(174), Protocol::Other(174));
    }

    #[test]
    fn protocol_from_ip_proto_175_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(175), Protocol::Other(175));
    }

    #[test]
    fn protocol_from_ip_proto_176_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(176), Protocol::Other(176));
    }

    #[test]
    fn protocol_from_ip_proto_177_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(177), Protocol::Other(177));
    }

    #[test]
    fn protocol_from_ip_proto_178_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(178), Protocol::Other(178));
    }

    #[test]
    fn protocol_from_ip_proto_179_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(179), Protocol::Other(179));
    }

    #[test]
    fn protocol_from_ip_proto_180_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(180), Protocol::Other(180));
    }

    #[test]
    fn protocol_from_ip_proto_181_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(181), Protocol::Other(181));
    }

    #[test]
    fn protocol_from_ip_proto_182_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(182), Protocol::Other(182));
    }

    #[test]
    fn protocol_from_ip_proto_183_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(183), Protocol::Other(183));
    }

    #[test]
    fn protocol_from_ip_proto_184_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(184), Protocol::Other(184));
    }

    #[test]
    fn protocol_from_ip_proto_185_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(185), Protocol::Other(185));
    }

    #[test]
    fn protocol_from_ip_proto_186_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(186), Protocol::Other(186));
    }

    #[test]
    fn protocol_from_ip_proto_187_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(187), Protocol::Other(187));
    }

    #[test]
    fn protocol_from_ip_proto_188_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(188), Protocol::Other(188));
    }

    #[test]
    fn protocol_from_ip_proto_189_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(189), Protocol::Other(189));
    }

    #[test]
    fn protocol_from_ip_proto_190_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(190), Protocol::Other(190));
    }

    #[test]
    fn protocol_from_ip_proto_191_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(191), Protocol::Other(191));
    }

    #[test]
    fn protocol_from_ip_proto_192_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(192), Protocol::Other(192));
    }

    #[test]
    fn protocol_from_ip_proto_193_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(193), Protocol::Other(193));
    }

    #[test]
    fn protocol_from_ip_proto_194_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(194), Protocol::Other(194));
    }

    #[test]
    fn protocol_from_ip_proto_195_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(195), Protocol::Other(195));
    }

    #[test]
    fn protocol_from_ip_proto_196_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(196), Protocol::Other(196));
    }

    #[test]
    fn protocol_from_ip_proto_197_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(197), Protocol::Other(197));
    }

    #[test]
    fn protocol_from_ip_proto_198_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(198), Protocol::Other(198));
    }

    #[test]
    fn protocol_from_ip_proto_199_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(199), Protocol::Other(199));
    }

    #[test]
    fn protocol_from_ip_proto_200_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(200), Protocol::Other(200));
    }

    #[test]
    fn protocol_from_ip_proto_201_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(201), Protocol::Other(201));
    }

    #[test]
    fn protocol_from_ip_proto_202_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(202), Protocol::Other(202));
    }

    #[test]
    fn protocol_from_ip_proto_203_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(203), Protocol::Other(203));
    }

    #[test]
    fn protocol_from_ip_proto_204_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(204), Protocol::Other(204));
    }

    #[test]
    fn protocol_from_ip_proto_205_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(205), Protocol::Other(205));
    }

    #[test]
    fn protocol_from_ip_proto_206_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(206), Protocol::Other(206));
    }

    #[test]
    fn protocol_from_ip_proto_207_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(207), Protocol::Other(207));
    }

    #[test]
    fn protocol_from_ip_proto_208_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(208), Protocol::Other(208));
    }

    #[test]
    fn protocol_from_ip_proto_209_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(209), Protocol::Other(209));
    }

    #[test]
    fn protocol_from_ip_proto_210_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(210), Protocol::Other(210));
    }

    #[test]
    fn protocol_from_ip_proto_211_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(211), Protocol::Other(211));
    }

    #[test]
    fn protocol_from_ip_proto_212_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(212), Protocol::Other(212));
    }

    #[test]
    fn protocol_from_ip_proto_213_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(213), Protocol::Other(213));
    }

    #[test]
    fn protocol_from_ip_proto_214_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(214), Protocol::Other(214));
    }

    #[test]
    fn protocol_from_ip_proto_215_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(215), Protocol::Other(215));
    }

    #[test]
    fn protocol_from_ip_proto_216_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(216), Protocol::Other(216));
    }

    #[test]
    fn protocol_from_ip_proto_217_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(217), Protocol::Other(217));
    }

    #[test]
    fn protocol_from_ip_proto_218_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(218), Protocol::Other(218));
    }

    #[test]
    fn protocol_from_ip_proto_219_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(219), Protocol::Other(219));
    }

    #[test]
    fn protocol_from_ip_proto_220_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(220), Protocol::Other(220));
    }

    #[test]
    fn protocol_from_ip_proto_221_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(221), Protocol::Other(221));
    }

    #[test]
    fn protocol_from_ip_proto_222_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(222), Protocol::Other(222));
    }

    #[test]
    fn protocol_from_ip_proto_223_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(223), Protocol::Other(223));
    }

    #[test]
    fn protocol_from_ip_proto_224_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(224), Protocol::Other(224));
    }

    #[test]
    fn protocol_from_ip_proto_225_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(225), Protocol::Other(225));
    }

    #[test]
    fn protocol_from_ip_proto_226_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(226), Protocol::Other(226));
    }

    #[test]
    fn protocol_from_ip_proto_227_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(227), Protocol::Other(227));
    }

    #[test]
    fn protocol_from_ip_proto_228_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(228), Protocol::Other(228));
    }

    #[test]
    fn protocol_from_ip_proto_229_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(229), Protocol::Other(229));
    }

    #[test]
    fn protocol_from_ip_proto_230_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(230), Protocol::Other(230));
    }

    #[test]
    fn protocol_from_ip_proto_231_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(231), Protocol::Other(231));
    }

    #[test]
    fn protocol_from_ip_proto_232_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(232), Protocol::Other(232));
    }

    #[test]
    fn protocol_from_ip_proto_233_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(233), Protocol::Other(233));
    }

    #[test]
    fn protocol_from_ip_proto_234_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(234), Protocol::Other(234));
    }

    #[test]
    fn protocol_from_ip_proto_235_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(235), Protocol::Other(235));
    }

    #[test]
    fn protocol_from_ip_proto_236_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(236), Protocol::Other(236));
    }

    #[test]
    fn protocol_from_ip_proto_237_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(237), Protocol::Other(237));
    }

    #[test]
    fn protocol_from_ip_proto_238_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(238), Protocol::Other(238));
    }

    #[test]
    fn protocol_from_ip_proto_239_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(239), Protocol::Other(239));
    }

    #[test]
    fn protocol_from_ip_proto_240_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(240), Protocol::Other(240));
    }

    #[test]
    fn protocol_from_ip_proto_241_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(241), Protocol::Other(241));
    }

    #[test]
    fn protocol_from_ip_proto_242_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(242), Protocol::Other(242));
    }

    #[test]
    fn protocol_from_ip_proto_243_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(243), Protocol::Other(243));
    }

    #[test]
    fn protocol_from_ip_proto_244_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(244), Protocol::Other(244));
    }

    #[test]
    fn protocol_from_ip_proto_245_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(245), Protocol::Other(245));
    }

    #[test]
    fn protocol_from_ip_proto_246_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(246), Protocol::Other(246));
    }

    #[test]
    fn protocol_from_ip_proto_247_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(247), Protocol::Other(247));
    }

    #[test]
    fn protocol_from_ip_proto_248_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(248), Protocol::Other(248));
    }

    #[test]
    fn protocol_from_ip_proto_249_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(249), Protocol::Other(249));
    }

    #[test]
    fn protocol_from_ip_proto_250_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(250), Protocol::Other(250));
    }

    #[test]
    fn protocol_from_ip_proto_251_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(251), Protocol::Other(251));
    }

    #[test]
    fn protocol_from_ip_proto_252_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(252), Protocol::Other(252));
    }

    #[test]
    fn protocol_from_ip_proto_253_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(253), Protocol::Other(253));
    }

    #[test]
    fn protocol_from_ip_proto_254_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(254), Protocol::Other(254));
    }

    #[test]
    fn protocol_from_ip_proto_255_maps_to_other() {
        assert_eq!(Protocol::from_ip_next_header(255), Protocol::Other(255));
    }
}
