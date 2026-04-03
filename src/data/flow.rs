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
}
