use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use dns_lookup::lookup_addr;

/// Maximum cache entries before eviction triggers.
const CACHE_HIGH_WATER: usize = 4096;
/// Entries older than this are eligible for eviction.
const CACHE_TTL_SECS: u64 = 300;
/// Maximum concurrent in-flight DNS lookups.
const MAX_PENDING: usize = 64;

/// Asynchronous DNS resolver with caching.
#[derive(Clone)]
pub struct Resolver {
    cache: Arc<Mutex<ResolverCache>>,
    enabled: bool,
}

#[derive(Debug)]
struct ResolverCache {
    entries: HashMap<IpAddr, CacheEntry>,
    pending_count: usize,
}

#[derive(Clone, Debug)]
struct CacheEntry {
    state: ResolveState,
    last_used: Instant,
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
            cache: Arc::new(Mutex::new(ResolverCache {
                entries: HashMap::new(),
                pending_count: 0,
            })),
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

        if let Some(entry) = cache.entries.get_mut(&addr) {
            entry.last_used = Instant::now();
            return match &entry.state {
                ResolveState::Resolved(name) => name.clone(),
                ResolveState::Pending | ResolveState::Failed => addr.to_string(),
            };
        }

        // Evict stale entries when cache grows too large
        if cache.entries.len() >= CACHE_HIGH_WATER {
            evict_stale(&mut cache);
        }

        // Cap concurrent DNS threads
        if cache.pending_count >= MAX_PENDING {
            return addr.to_string();
        }

        // Start background resolution
        let now = Instant::now();
        cache.entries.insert(
            addr,
            CacheEntry {
                state: ResolveState::Pending,
                last_used: now,
            },
        );
        cache.pending_count += 1;
        let cache_ref = Arc::clone(&self.cache);
        std::thread::spawn(move || {
            let result = lookup_addr(&addr);
            let mut cache = cache_ref.lock().unwrap_or_else(|e| e.into_inner());
            cache.pending_count = cache.pending_count.saturating_sub(1);
            let state = match result {
                Ok(hostname) => ResolveState::Resolved(hostname),
                Err(_) => ResolveState::Failed,
            };
            if let Some(entry) = cache.entries.get_mut(&addr) {
                entry.state = state;
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

/// Remove entries not used within the TTL window. If the cache is still above
/// the high-water mark after TTL eviction, drop the oldest half.
fn evict_stale(cache: &mut ResolverCache) {
    let cutoff = Instant::now() - std::time::Duration::from_secs(CACHE_TTL_SECS);
    cache.entries.retain(|_, entry| entry.last_used > cutoff);

    // If still over capacity, keep only the most recently used half
    if cache.entries.len() >= CACHE_HIGH_WATER {
        let mut by_age: Vec<(IpAddr, Instant)> = cache
            .entries
            .iter()
            .map(|(ip, e)| (*ip, e.last_used))
            .collect();
        by_age.sort_by(|a, b| b.1.cmp(&a.1)); // newest first
        let keep: usize = CACHE_HIGH_WATER / 2;
        let to_remove: Vec<IpAddr> = by_age.iter().skip(keep).map(|(ip, _)| *ip).collect();
        for ip in to_remove {
            cache.entries.remove(&ip);
        }
    }

    // Recount pending entries after eviction
    cache.pending_count = cache
        .entries
        .values()
        .filter(|e| matches!(e.state, ResolveState::Pending))
        .count();
}

// ─── /etc/services lookup ─────────────────────────────────────────────────────

/// Parsed entry from /etc/services: maps (port, protocol) -> service name.
type ServicesMap = HashMap<(u16, &'static str), &'static str>;

/// Lazily parsed, globally cached /etc/services.
fn services_map() -> &'static ServicesMap {
    static MAP: OnceLock<ServicesMap> = OnceLock::new();
    MAP.get_or_init(parse_etc_services_file)
}

/// Normalize `tcp` / `udp` keys so lookups match files that use `TCP` / `UDP`.
fn normalize_protocol(proto: &str) -> &'static str {
    if proto.eq_ignore_ascii_case("tcp") {
        "tcp"
    } else if proto.eq_ignore_ascii_case("udp") {
        "udp"
    } else {
        Box::leak(proto.to_ascii_lowercase().into_boxed_str())
    }
}

fn parse_etc_services_file() -> ServicesMap {
    let contents = match std::fs::read_to_string("/etc/services") {
        Ok(s) => s,
        Err(_) => return ServicesMap::new(),
    };
    let contents: &'static str = Box::leak(contents.into_boxed_str());
    parse_etc_services_text(contents)
}

/// Parse services(5)-style lines. `contents` must be `'static` so service names
/// can live in a leaked buffer or `include_str!` data.
fn parse_etc_services_text(contents: &'static str) -> ServicesMap {
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
        let proto_raw = match pp_split.next() {
            Some(p) => p,
            None => continue,
        };
        let port: u16 = match port_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };
        let proto = normalize_protocol(proto_raw);
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
    use std::sync::OnceLock;

    /// Canonical service names for assertions (see `tests/fixtures/minimal_etc_services.txt`).
    static FIXTURE_MAP: OnceLock<ServicesMap> = OnceLock::new();

    fn fixture_services_map() -> &'static ServicesMap {
        FIXTURE_MAP.get_or_init(|| {
            parse_etc_services_text(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/minimal_etc_services.txt"
            )))
        })
    }

    fn fixture_port_to_service(port: u16, tcp: bool) -> Option<&'static str> {
        let proto = if tcp { "tcp" } else { "udp" };
        fixture_services_map().get(&(port, proto)).copied()
    }

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

    // ── parse_etc_services_text (deterministic) ──

    #[test]
    fn parse_etc_services_text_empty_yields_empty_map() {
        let m = parse_etc_services_text("");
        assert!(m.is_empty());
    }

    #[test]
    fn parse_etc_services_text_skips_comments_and_blanks() {
        let m = parse_etc_services_text("# header\n\n  \nfoo 7/tcp\n# trailing\nbar 7/udp\n");
        assert_eq!(m.get(&(7, "tcp")).copied(), Some("foo"));
        assert_eq!(m.get(&(7, "udp")).copied(), Some("bar"));
    }

    #[test]
    fn parse_etc_services_text_normalizes_protocol_casing() {
        let m = parse_etc_services_text("www 80/TCP\n");
        assert_eq!(m.get(&(80, "tcp")).copied(), Some("www"));
    }

    #[test]
    fn parse_etc_services_text_first_line_wins_same_port_proto() {
        let m = parse_etc_services_text("a 80/tcp\nb 80/tcp\n");
        assert_eq!(m.get(&(80, "tcp")).copied(), Some("a"));
    }

    #[test]
    fn parse_etc_services_text_skips_non_numeric_port() {
        let m = parse_etc_services_text("bad abc/tcp\n");
        assert!(m.is_empty());
    }

    #[test]
    fn parse_etc_services_text_skips_line_without_slash_in_port_field() {
        let m = parse_etc_services_text("echo 7 tcp\n");
        assert!(m.is_empty());
    }

    #[test]
    fn parse_etc_services_text_skips_single_token_line() {
        let m = parse_etc_services_text("onlyname\n");
        assert!(m.is_empty());
    }

    #[test]
    fn parse_etc_services_text_accepts_tabs_between_fields() {
        let m = parse_etc_services_text("echo\t7/tcp\n");
        assert_eq!(m.get(&(7, "tcp")).copied(), Some("echo"));
    }

    #[test]
    fn parse_etc_services_text_udp_lowercase_explicit() {
        let m = parse_etc_services_text("domain 53/udp\n");
        assert_eq!(m.get(&(53, "udp")).copied(), Some("domain"));
    }

    #[test]
    fn parse_etc_services_text_port_max_u16() {
        let m = parse_etc_services_text("hi 65535/tcp\n");
        assert_eq!(m.get(&(65535, "tcp")).copied(), Some("hi"));
    }

    #[test]
    fn parse_etc_services_text_port_zero_parses() {
        let m = parse_etc_services_text("reserved 0/tcp\n");
        assert_eq!(m.get(&(0, "tcp")).copied(), Some("reserved"));
    }

    #[test]
    fn parse_etc_services_text_non_tcp_udp_protocol_lowercased() {
        let m = parse_etc_services_text("sctp-svc 2905/SCTP\n");
        assert_eq!(m.get(&(2905, "sctp")).copied(), Some("sctp-svc"));
    }

    #[test]
    fn parse_etc_services_text_multiple_spaces_between_columns() {
        let m = parse_etc_services_text("ssh     22/tcp\n");
        assert_eq!(m.get(&(22, "tcp")).copied(), Some("ssh"));
    }

    #[test]
    fn parse_etc_services_text_trailing_comment_after_aliases_ignored_as_fields() {
        // Third whitespace field is an alias, not a comment — still one port/proto pair
        let m = parse_etc_services_text("www 80/tcp www-http\n");
        assert_eq!(m.get(&(80, "tcp")).copied(), Some("www"));
    }

    #[test]
    fn parse_etc_services_text_crlf_line_endings() {
        let m = parse_etc_services_text("a 1/tcp\r\nb 2/udp\r\n");
        assert_eq!(m.get(&(1, "tcp")).copied(), Some("a"));
        assert_eq!(m.get(&(2, "udp")).copied(), Some("b"));
    }

    #[test]
    fn parse_etc_services_text_only_comment_lines() {
        let m = parse_etc_services_text("# a\n# b\n");
        assert!(m.is_empty());
    }

    #[test]
    fn parse_etc_services_text_mixed_leading_whitespace() {
        let m = parse_etc_services_text("  \t  ssh 22/tcp\n");
        assert_eq!(m.get(&(22, "tcp")).copied(), Some("ssh"));
    }

    #[test]
    fn parse_etc_services_text_duplicate_udp_and_tcp_distinct_keys() {
        let m = parse_etc_services_text("a 7/tcp\nb 7/udp\n");
        assert_eq!(m.get(&(7, "tcp")).copied(), Some("a"));
        assert_eq!(m.get(&(7, "udp")).copied(), Some("b"));
    }

    #[test]
    fn parse_etc_services_text_extra_slashes_after_proto_use_first_two_segments() {
        let m = parse_etc_services_text("x 99/tcp/extra\n");
        assert_eq!(m.get(&(99, "tcp")).copied(), Some("x"));
    }

    #[test]
    fn parse_etc_services_text_preserves_hyphenated_service_name() {
        let m = parse_etc_services_text("my-service 8080/tcp\n");
        assert_eq!(m.get(&(8080, "tcp")).copied(), Some("my-service"));
    }

    #[test]
    fn parse_etc_services_text_numeric_service_name() {
        let m = parse_etc_services_text("12345 9/tcp\n");
        assert_eq!(m.get(&(9, "tcp")).copied(), Some("12345"));
    }

    #[test]
    fn parse_etc_services_text_leading_hash_skips_entire_line() {
        let m = parse_etc_services_text("#ssh 22/tcp\n");
        assert!(m.is_empty());
    }

    #[test]
    fn parse_etc_services_text_blank_line_between_entries() {
        let m = parse_etc_services_text("a 1/tcp\n\nb 2/udp\n");
        assert_eq!(m.get(&(1, "tcp")).copied(), Some("a"));
        assert_eq!(m.get(&(2, "udp")).copied(), Some("b"));
    }

    #[test]
    fn fixture_port_to_service_distinct_tcp_udp_same_port_number() {
        assert_eq!(fixture_port_to_service(25, true), Some("smtp"));
        assert_eq!(fixture_port_to_service(25, false), Some("smtp"));
    }

    #[test]
    fn parse_etc_services_text_udp_uppercase_normalized() {
        let m = parse_etc_services_text("dns 53/UDP\n");
        assert_eq!(m.get(&(53, "udp")).copied(), Some("dns"));
    }

    #[test]
    fn fixture_map_contains_nntp_and_imap_fixture_names() {
        assert_eq!(fixture_port_to_service(119, true), Some("nntp"));
        assert_eq!(fixture_port_to_service(143, true), Some("imap"));
    }

    #[test]
    fn parse_etc_services_text_dotted_service_name() {
        let m = parse_etc_services_text("svc.name 12345/tcp\n");
        assert_eq!(m.get(&(12345, "tcp")).copied(), Some("svc.name"));
    }

    #[test]
    fn parse_etc_services_text_underscore_service_name() {
        let m = parse_etc_services_text("my_svc 9000/udp\n");
        assert_eq!(m.get(&(9000, "udp")).copied(), Some("my_svc"));
    }

    #[test]
    fn fixture_port_to_service_ntp_udp_fixture() {
        assert_eq!(fixture_port_to_service(123, false), Some("ntp"));
    }

    #[test]
    fn fixture_port_to_service_https_udp_fixture() {
        assert_eq!(fixture_port_to_service(443, false), Some("https"));
    }

    #[test]
    fn parse_etc_services_text_utf8_service_name_bytes() {
        let m = parse_etc_services_text("café 9/tcp\n");
        assert_eq!(m.get(&(9, "tcp")).copied(), Some("café"));
    }

    #[test]
    fn parse_etc_services_text_plus_in_service_name() {
        let m = parse_etc_services_text("svc+alt 8080/tcp\n");
        assert_eq!(m.get(&(8080, "tcp")).copied(), Some("svc+alt"));
    }

    #[test]
    fn parse_etc_services_text_trims_leading_line_whitespace_before_name() {
        let m = parse_etc_services_text("\t  ssh 22/tcp\n");
        assert_eq!(m.get(&(22, "tcp")).copied(), Some("ssh"));
    }

    #[test]
    fn parse_etc_services_text_skips_line_with_port_above_u16_max() {
        let m = parse_etc_services_text("bad 65536/tcp\n");
        assert!(m.is_empty());
    }

    #[test]
    fn parse_etc_services_text_allows_trailing_comment_after_port_proto() {
        let m = parse_etc_services_text("ftp 21/tcp # control\n");
        assert_eq!(m.get(&(21, "tcp")).copied(), Some("ftp"));
    }

    #[test]
    fn parse_etc_services_text_trailing_comment_can_contain_hash_token() {
        let m = parse_etc_services_text("ssh 22/tcp   # backup also uses #2222\n");
        assert_eq!(m.get(&(22, "tcp")).copied(), Some("ssh"));
    }

    #[test]
    fn parse_etc_services_text_leading_zero_port_parses_as_decimal() {
        let m = parse_etc_services_text("ssh 022/tcp\n");
        assert_eq!(m.get(&(22, "tcp")).copied(), Some("ssh"));
    }

    #[test]
    fn parse_etc_services_text_skips_line_with_empty_port_proto_token() {
        let m = parse_etc_services_text("ssh /tcp\n");
        assert!(m.is_empty());
    }

    #[test]
    fn parse_etc_services_text_skips_whitespace_only_line_between_entries() {
        let m = parse_etc_services_text("a 1/tcp\n   \n\t\nb 2/udp\n");
        assert_eq!(m.get(&(1, "tcp")).copied(), Some("a"));
        assert_eq!(m.get(&(2, "udp")).copied(), Some("b"));
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn parse_etc_services_text_comma_inside_service_name_parses() {
        let m = parse_etc_services_text("svc,alias 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc,alias"));
    }

    #[test]
    fn parse_etc_services_text_preserves_uppercase_service_name() {
        let m = parse_etc_services_text("SSH 22/tcp\n");
        assert_eq!(m.get(&(22, "tcp")).copied(), Some("SSH"));
    }

    #[test]
    fn parse_etc_services_text_skips_line_with_non_integer_port_token() {
        let m = parse_etc_services_text("bad 22.5/tcp\n");
        assert!(m.is_empty());
    }

    #[test]
    fn parse_etc_services_text_indented_full_line_comment_skipped() {
        let m = parse_etc_services_text("    # not-a-service 99/tcp\nssh 22/tcp\n");
        assert_eq!(m.get(&(22, "tcp")).copied(), Some("ssh"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_extra_spaces_between_name_and_port_field() {
        let m = parse_etc_services_text("ssh\t  22/tcp\n");
        assert_eq!(m.get(&(22, "tcp")).copied(), Some("ssh"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_leading_blank_line_then_entry() {
        let m = parse_etc_services_text("\nssh 22/tcp\n");
        assert_eq!(m.get(&(22, "tcp")).copied(), Some("ssh"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_at_sign_in_service_name_token() {
        let m = parse_etc_services_text("user@host 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("user@host"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_colon_inside_service_name_token() {
        let m = parse_etc_services_text("svc:alias 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc:alias"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_exclamation_in_service_name_token() {
        let m = parse_etc_services_text("svc! 9/tcp\n");
        assert_eq!(m.get(&(9, "tcp")).copied(), Some("svc!"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_ampersand_in_service_name_token() {
        let m = parse_etc_services_text("svc&more 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc&more"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_tilde_in_service_name_token() {
        let m = parse_etc_services_text("svc~bak 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc~bak"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_dollar_in_service_name_token() {
        let m = parse_etc_services_text("svc$name 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc$name"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_parentheses_in_service_name_token() {
        let m = parse_etc_services_text("svc(x) 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc(x)"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_square_brackets_in_service_name_token() {
        let m = parse_etc_services_text("svc[alt] 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc[alt]"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_braces_in_service_name_token() {
        let m = parse_etc_services_text("svc{alt} 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc{alt}"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_percent_in_service_name_token() {
        let m = parse_etc_services_text("svc%enc 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc%enc"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_apostrophe_in_service_name_token() {
        let m = parse_etc_services_text("svc'alt 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc'alt"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_backslash_in_service_name_token() {
        let m = parse_etc_services_text("svc\\net 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc\\net"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_asterisk_in_service_name_token() {
        let m = parse_etc_services_text("svc*wild 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc*wild"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_equals_in_service_name_token() {
        let m = parse_etc_services_text("svc=x 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc=x"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_pipe_in_service_name_token() {
        let m = parse_etc_services_text("a|b 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("a|b"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_semicolon_in_service_name_token() {
        let m = parse_etc_services_text("svc;meta 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc;meta"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_caret_in_service_name_token() {
        let m = parse_etc_services_text("svc^pat 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc^pat"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_question_mark_in_service_name_token() {
        let m = parse_etc_services_text("svc?opt 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc?opt"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_angle_brackets_in_service_name_token() {
        let m = parse_etc_services_text("svc<alt> 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc<alt>"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_backtick_in_service_name_token() {
        let m = parse_etc_services_text("svc`x` 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc`x`"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_hash_inside_service_name_token() {
        let m = parse_etc_services_text("svc#int 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc#int"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_euro_in_service_name_token() {
        let m = parse_etc_services_text("svc\u{20AC} 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc\u{20AC}"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_en_dash_in_service_name_token() {
        let m = parse_etc_services_text("svc\u{2013}alt 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc\u{2013}alt"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_em_dash_in_service_name_token() {
        let m = parse_etc_services_text("svc\u{2014}alt 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc\u{2014}alt"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_middle_dot_in_service_name_token() {
        // U+00A0 is Unicode whitespace in Rust (`char::is_whitespace`), so `split_whitespace`
        // would break the service name across tokens — use U+00B7 (middle dot), not a space.
        let m = parse_etc_services_text("svc\u{00B7}name 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc\u{00B7}name"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_soft_hyphen_in_service_name_token() {
        let m = parse_etc_services_text("svc\u{00AD}name 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc\u{00AD}name"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_word_joiner_in_service_name_token() {
        let m = parse_etc_services_text("svc\u{2060}z 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc\u{2060}z"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_rtl_mark_in_service_name_token() {
        let m = parse_etc_services_text("svc\u{200F}z 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc\u{200F}z"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_superscript_two_in_service_name_token() {
        let m = parse_etc_services_text("svc\u{00B2}name 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc\u{00B2}name"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_variation_selector_in_service_name_token() {
        let m = parse_etc_services_text("svc\u{FE0F}z 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc\u{FE0F}z"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_zero_width_joiner_in_service_name_token() {
        // U+200D is not Unicode whitespace — it stays inside the first `split_whitespace` token.
        let m = parse_etc_services_text("svc\u{200D}z 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc\u{200D}z"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_greek_alpha_in_service_name_token() {
        let m = parse_etc_services_text("svc\u{03B1}port 2222/tcp\n");
        assert_eq!(m.get(&(2222, "tcp")).copied(), Some("svc\u{03B1}port"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_fullwidth_digits_in_port_token_skips_line() {
        // U+FF10–U+FF19 are not ASCII digits — `u16` parse skips the line.
        let m = parse_etc_services_text("bad \u{FF18}\u{FF10}/tcp\nssh 22/tcp\n");
        assert_eq!(m.get(&(22, "tcp")).copied(), Some("ssh"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_arabic_indic_digits_in_port_token_skips_line() {
        // Arabic-Indic digits (U+0660–U+0669) are not ASCII — port parse fails.
        let m = parse_etc_services_text("bad \u{0661}\u{0662}/tcp\ndomain 53/udp\n");
        assert_eq!(m.get(&(53, "udp")).copied(), Some("domain"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn parse_etc_services_text_skips_line_with_negative_port_token() {
        let m = parse_etc_services_text("bad -1/tcp\n");
        assert!(m.is_empty());
    }

    #[test]
    fn parse_etc_services_text_port_65535_valid() {
        let m = parse_etc_services_text("maxp 65535/tcp\n");
        assert_eq!(m.get(&(65535, "tcp")).copied(), Some("maxp"));
    }

    #[test]
    fn parse_etc_services_text_single_line_without_trailing_newline() {
        let m = parse_etc_services_text("echo 7/tcp");
        assert_eq!(m.get(&(7, "tcp")).copied(), Some("echo"));
    }

    #[test]
    fn parse_etc_services_text_slash_in_service_name_token() {
        let m = parse_etc_services_text("a/b 99/tcp\n");
        assert_eq!(m.get(&(99, "tcp")).copied(), Some("a/b"));
    }

    #[test]
    fn parse_etc_services_text_two_character_service_name() {
        let m = parse_etc_services_text("me 9/tcp\n");
        assert_eq!(m.get(&(9, "tcp")).copied(), Some("me"));
    }

    #[test]
    fn parse_etc_services_text_line_with_only_hash_is_skipped() {
        let m = parse_etc_services_text("# full line comment only\nssh 22/tcp\n");
        assert_eq!(m.get(&(22, "tcp")).copied(), Some("ssh"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn fixture_map_lists_expected_well_known_ports() {
        let m = fixture_services_map();
        assert!(m.len() >= 12);
        assert_eq!(fixture_port_to_service(25, true), Some("smtp"));
        assert_eq!(fixture_port_to_service(25, false), Some("smtp"));
        assert_eq!(fixture_port_to_service(80, true), Some("http"));
        assert_eq!(fixture_port_to_service(80, false), Some("http"));
        assert_eq!(fixture_port_to_service(443, true), Some("https"));
        assert_eq!(fixture_port_to_service(443, false), Some("https"));
        assert_eq!(fixture_port_to_service(22, true), Some("ssh"));
        assert_eq!(fixture_port_to_service(53, true), Some("domain"));
        assert_eq!(fixture_port_to_service(53, false), Some("domain"));
        assert_eq!(fixture_port_to_service(23, true), Some("telnet"));
        assert_eq!(fixture_port_to_service(21, true), Some("ftp"));
        assert_eq!(fixture_port_to_service(110, true), Some("pop3"));
        assert_eq!(fixture_port_to_service(123, false), Some("ntp"));
        assert_eq!(fixture_port_to_service(143, true), Some("imap"));
        assert_eq!(fixture_port_to_service(119, true), Some("nntp"));
    }

    // ── port_to_service (real /etc/services smoke tests) ──

    #[test]
    fn port_to_service_os_lists_http_80_tcp() {
        if services_map().is_empty() {
            // No readable /etc/services (e.g. Windows, minimal containers).
            return;
        }
        assert!(
            port_to_service(80, true).is_some(),
            "expected port 80/tcp in system /etc/services"
        );
    }

    #[test]
    fn port_to_service_unknown_port() {
        let result = port_to_service(65432, true);
        assert!(result.is_none());
    }

    #[test]
    fn port_to_service_zero() {
        let result = port_to_service(0, true);
        let _ = result;
    }

    // ── services_map ──

    #[test]
    fn services_map_is_populated() {
        let map = services_map();
        if map.is_empty() {
            return;
        }
        // Should have at least a few standard entries when /etc/services is present
        assert!(map.len() > 10);
    }

    #[test]
    fn services_map_same_instance() {
        let a = services_map() as *const _;
        let b = services_map() as *const _;
        assert_eq!(a, b, "services_map should return same OnceLock instance");
    }

    // ── Cache eviction ──

    #[test]
    fn evict_stale_removes_old_entries() {
        let mut cache = ResolverCache {
            entries: HashMap::new(),
            pending_count: 0,
        };
        let old = Instant::now() - std::time::Duration::from_secs(CACHE_TTL_SECS + 10);
        let recent = Instant::now();

        let old_ip: IpAddr = "10.0.0.1".parse().unwrap();
        let new_ip: IpAddr = "10.0.0.2".parse().unwrap();

        cache.entries.insert(
            old_ip,
            CacheEntry {
                state: ResolveState::Failed,
                last_used: old,
            },
        );
        cache.entries.insert(
            new_ip,
            CacheEntry {
                state: ResolveState::Resolved("host".into()),
                last_used: recent,
            },
        );

        evict_stale(&mut cache);
        assert!(!cache.entries.contains_key(&old_ip));
        assert!(cache.entries.contains_key(&new_ip));
    }

    #[test]
    fn evict_stale_halves_when_all_recent() {
        let mut cache = ResolverCache {
            entries: HashMap::new(),
            pending_count: 0,
        };
        let now = Instant::now();

        // Fill to high-water mark with recent entries
        for i in 0..CACHE_HIGH_WATER {
            let ip: IpAddr = format!("10.{}.{}.{}", (i >> 16) & 0xFF, (i >> 8) & 0xFF, i & 0xFF)
                .parse()
                .unwrap();
            cache.entries.insert(
                ip,
                CacheEntry {
                    state: ResolveState::Resolved(format!("host{}", i)),
                    last_used: now,
                },
            );
        }

        evict_stale(&mut cache);
        assert!(cache.entries.len() <= CACHE_HIGH_WATER / 2);
    }

    #[test]
    fn pending_cap_prevents_excessive_threads() {
        let r = Resolver::new(true);
        {
            let mut cache = r.cache.lock().unwrap();
            cache.pending_count = MAX_PENDING;
        }
        // With pending at cap, resolve should return IP without spawning
        let addr: IpAddr = "203.0.113.1".parse().unwrap();
        let result = r.resolve(addr);
        assert_eq!(result, "203.0.113.1");
        // No entry should have been inserted
        let cache = r.cache.lock().unwrap();
        assert!(!cache.entries.contains_key(&addr));
    }

    #[test]
    fn evict_stale_empty_cache_no_panic() {
        let mut cache = ResolverCache {
            entries: HashMap::new(),
            pending_count: 0,
        };
        evict_stale(&mut cache);
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn resolver_new_enabled() {
        let r = Resolver::new(true);
        assert!(r.is_enabled());
    }

    #[test]
    fn resolver_new_disabled() {
        let r = Resolver::new(false);
        assert!(!r.is_enabled());
    }

    #[test]
    fn resolver_ipv6_enabled_returns_literal() {
        let r = Resolver::new(false);
        let addr: IpAddr = "2001:db8::1".parse().unwrap();
        assert_eq!(r.resolve(addr), "2001:db8::1");
    }

    #[test]
    fn services_map_contains_ssh() {
        let m = services_map();
        if m.is_empty() {
            return;
        }
        assert!(m.contains_key(&(22, "tcp")));
    }

    #[test]
    fn resolve_multiple_ips_no_panic() {
        let r = Resolver::new(false);
        for s in ["1.1.1.1", "8.8.8.8", "::1"] {
            let addr: IpAddr = s.parse().unwrap();
            let _ = r.resolve(addr);
        }
    }

    #[test]
    fn port_to_service_high_ephemeral_port_returns_none() {
        assert!(port_to_service(49152, true).is_none());
    }
}
