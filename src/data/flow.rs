use std::fmt;
use std::net::IpAddr;

/// Uniquely identifies a network flow (bidirectional).
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
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
        let k2 = k1.clone();
        assert_eq!(k1, k2);
    }

    #[test]
    fn flow_key_inequality_port() {
        let k1 = FlowKey { src: "10.0.0.1".parse().unwrap(), dst: "10.0.0.2".parse().unwrap(), src_port: 80, dst_port: 80, protocol: Protocol::Tcp };
        let k2 = FlowKey { src: "10.0.0.1".parse().unwrap(), dst: "10.0.0.2".parse().unwrap(), src_port: 81, dst_port: 80, protocol: Protocol::Tcp };
        assert_ne!(k1, k2);
    }

    #[test]
    fn flow_key_inequality_protocol() {
        let k1 = FlowKey { src: "10.0.0.1".parse().unwrap(), dst: "10.0.0.2".parse().unwrap(), src_port: 80, dst_port: 80, protocol: Protocol::Tcp };
        let k2 = FlowKey { src: "10.0.0.1".parse().unwrap(), dst: "10.0.0.2".parse().unwrap(), src_port: 80, dst_port: 80, protocol: Protocol::Udp };
        assert_ne!(k1, k2);
    }

    #[test]
    fn flow_key_inequality_ip() {
        let k1 = FlowKey { src: "10.0.0.1".parse().unwrap(), dst: "10.0.0.2".parse().unwrap(), src_port: 80, dst_port: 80, protocol: Protocol::Tcp };
        let k2 = FlowKey { src: "10.0.0.3".parse().unwrap(), dst: "10.0.0.2".parse().unwrap(), src_port: 80, dst_port: 80, protocol: Protocol::Tcp };
        assert_ne!(k1, k2);
    }

    #[test]
    fn flow_key_hash_consistency() {
        use std::collections::HashMap;
        let k1 = FlowKey { src: "10.0.0.1".parse().unwrap(), dst: "10.0.0.2".parse().unwrap(), src_port: 80, dst_port: 443, protocol: Protocol::Tcp };
        let k2 = k1.clone();
        let mut map = HashMap::new();
        map.insert(k1, 42);
        assert_eq!(map.get(&k2), Some(&42));
    }

    #[test]
    fn flow_key_ipv6() {
        let k = FlowKey { src: "::1".parse().unwrap(), dst: "::2".parse().unwrap(), src_port: 80, dst_port: 443, protocol: Protocol::Tcp };
        let k2 = k.clone();
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
}
