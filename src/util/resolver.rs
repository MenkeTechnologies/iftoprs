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

        let mut cache = self.cache.lock().unwrap();
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
            let mut cache = cache_ref.lock().unwrap();
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
