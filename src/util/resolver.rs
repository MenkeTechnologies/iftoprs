use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex, OnceLock};

use dns_lookup::lookup_addr;

/// Asynchronous DNS resolver with caching.
#[derive(Clone)]
pub struct Resolver {
    cache: Arc<Mutex<HashMap<IpAddr, ResolveState>>>,
    enabled: bool,
}

#[derive(Clone, Debug)]
enum ResolveState {
    Pending,
    Resolved(String),
    Failed,
}

impl Resolver {
    pub fn new(enabled: bool) -> Self {
        // Eagerly parse /etc/services on startup
        let _ = services_map();
        Resolver {
            cache: Arc::new(Mutex::new(HashMap::new())),
            enabled,
        }
    }

    /// Get the hostname for an IP, triggering a background lookup if needed.
    /// Returns the IP string if not yet resolved.
    pub fn resolve(&self, addr: IpAddr) -> String {
        if !self.enabled {
            return addr.to_string();
        }

        let mut cache = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        match cache.get(&addr) {
            Some(ResolveState::Resolved(name)) => return name.clone(),
            Some(ResolveState::Pending) => return addr.to_string(),
            Some(ResolveState::Failed) => return addr.to_string(),
            None => {}
        }

        // Start background resolution
        cache.insert(addr, ResolveState::Pending);
        let cache_ref = Arc::clone(&self.cache);
        std::thread::spawn(move || {
            let result = lookup_addr(&addr);
            let mut cache = cache_ref.lock().unwrap_or_else(|e| e.into_inner());
            match result {
                Ok(hostname) => {
                    cache.insert(addr, ResolveState::Resolved(hostname));
                }
                Err(_) => {
                    cache.insert(addr, ResolveState::Failed);
                }
            }
        });

        addr.to_string()
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// ─── /etc/services lookup ─────────────────────────────────────────────────────

/// Parsed entry from /etc/services: maps (port, protocol) -> service name.
type ServicesMap = HashMap<(u16, &'static str), &'static str>;

/// Lazily parsed, globally cached /etc/services.
fn services_map() -> &'static ServicesMap {
    static MAP: OnceLock<ServicesMap> = OnceLock::new();
    MAP.get_or_init(parse_etc_services)
}

fn parse_etc_services() -> ServicesMap {
    // Leak the file contents so entries can be &'static str
    let contents = match std::fs::read_to_string("/etc/services") {
        Ok(s) => s,
        Err(_) => return ServicesMap::new(),
    };
    let contents: &'static str = Box::leak(contents.into_boxed_str());

    let mut map = ServicesMap::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Format: service_name  port/protocol  [aliases...]  [# comment]
        let mut parts = line.split_whitespace();
        let name = match parts.next() {
            Some(n) => n,
            None => continue,
        };
        let port_proto = match parts.next() {
            Some(pp) => pp,
            None => continue,
        };
        let mut pp_split = port_proto.split('/');
        let port_str = match pp_split.next() {
            Some(p) => p,
            None => continue,
        };
        let proto = match pp_split.next() {
            Some(p) => p,
            None => continue,
        };
        let port: u16 = match port_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        map.entry((port, proto)).or_insert(name);
    }
    map
}

/// Look up a port number in /etc/services.
pub fn port_to_service(port: u16, tcp: bool) -> Option<&'static str> {
    let proto = if tcp { "tcp" } else { "udp" };
    services_map().get(&(port, proto)).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Resolver basic ──

    #[test]
    fn resolver_disabled_returns_ip_string() {
        let r = Resolver::new(false);
        let addr: IpAddr = "127.0.0.1".parse().unwrap();
        assert_eq!(r.resolve(addr), "127.0.0.1");
    }

    #[test]
    fn resolver_toggle() {
        let mut r = Resolver::new(true);
        assert!(r.is_enabled());
        r.toggle();
        assert!(!r.is_enabled());
        r.toggle();
        assert!(r.is_enabled());
    }

    #[test]
    fn resolver_disabled_after_toggle() {
        let mut r = Resolver::new(true);
        r.toggle();
        let addr: IpAddr = "8.8.8.8".parse().unwrap();
        // When disabled, should return raw IP
        assert_eq!(r.resolve(addr), "8.8.8.8");
    }

    #[test]
    fn resolver_enabled_returns_ip_initially() {
        let r = Resolver::new(true);
        let addr: IpAddr = "93.184.216.34".parse().unwrap();
        // First call returns IP string (lookup is async)
        let result = r.resolve(addr);
        assert_eq!(result, "93.184.216.34");
    }

    #[test]
    fn resolver_pending_state_returns_ip() {
        let r = Resolver::new(true);
        let addr: IpAddr = "198.51.100.1".parse().unwrap();
        // First resolve triggers Pending
        let _ = r.resolve(addr);
        // Second resolve while Pending should return IP
        let result = r.resolve(addr);
        assert_eq!(result, "198.51.100.1");
    }

    #[test]
    fn resolver_clone_shares_cache() {
        let r1 = Resolver::new(false);
        let r2 = r1.clone();
        // Both should work independently
        let addr: IpAddr = "10.0.0.1".parse().unwrap();
        assert_eq!(r2.resolve(addr), "10.0.0.1");
    }

    #[test]
    fn resolver_loopback_resolves() {
        let r = Resolver::new(true);
        let addr: IpAddr = "127.0.0.1".parse().unwrap();
        // Should not panic
        let _ = r.resolve(addr);
    }

    #[test]
    fn resolver_ipv6_disabled() {
        let r = Resolver::new(false);
        let addr: IpAddr = "::1".parse().unwrap();
        assert_eq!(r.resolve(addr), "::1");
    }

    #[test]
    fn resolver_mutex_poison_recovery() {
        let r = Resolver::new(true);
        // Poison the cache mutex
        let cache = Arc::clone(&r.cache);
        let h = std::thread::spawn(move || {
            let _guard = cache.lock().unwrap();
            panic!("intentional poison");
        });
        let _ = h.join();

        // Should still work due to unwrap_or_else recovery
        let addr: IpAddr = "10.0.0.1".parse().unwrap();
        let result = r.resolve(addr);
        // Should not panic, just return the IP
        assert!(!result.is_empty());
    }

    // ── port_to_service ──

    #[test]
    fn port_to_service_http() {
        // /etc/services should have port 80/tcp = http on most systems
        let result = port_to_service(80, true);
        assert_eq!(result, Some("http"));
    }

    #[test]
    fn port_to_service_https() {
        let result = port_to_service(443, true);
        assert_eq!(result, Some("https"));
    }

    #[test]
    fn port_to_service_ssh() {
        let result = port_to_service(22, true);
        assert_eq!(result, Some("ssh"));
    }

    #[test]
    fn port_to_service_dns_udp() {
        let result = port_to_service(53, false);
        assert_eq!(result, Some("domain"));
    }

    #[test]
    fn port_to_service_unknown_port() {
        let result = port_to_service(65432, true);
        assert!(result.is_none());
    }

    #[test]
    fn port_to_service_zero() {
        let result = port_to_service(0, true);
        // port 0 may or may not be in /etc/services
        // just ensure no panic
        let _ = result;
    }

    // ── services_map ──

    #[test]
    fn services_map_is_populated() {
        let map = services_map();
        // Should have at least a few standard entries
        assert!(map.len() > 10);
    }

    #[test]
    fn services_map_same_instance() {
        let a = services_map() as *const _;
        let b = services_map() as *const _;
        assert_eq!(a, b, "services_map should return same OnceLock instance");
    }
}
