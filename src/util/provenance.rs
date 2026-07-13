//! Provenance attribution — resolves a live PID past its process *name* to the
//! *code identity* that produced the binary on disk. This is the third
//! aggregation axis (Publishers) alongside Flows and Processes: it answers
//! "which vendor / package is on my uplink", not just "which process".
//!
//! - macOS: the code-signing Team Identifier plus a signing-authority verdict,
//!   read from the on-disk Mach-O through the Security framework
//!   (`SecStaticCodeCreateWithPath` + `SecCodeCopySigningInformation`, plus
//!   `anchor apple` / `anchor apple generic` requirement probes).
//! - Linux: the owning distribution package (dpkg/rpm) plus the SHA-256 of the
//!   executable image. Pure std + the package tools; no extra crate.
//!
//! Resolution is cached keyed by the executable's `(dev, inode, mtime)` so any
//! one binary is fingerprinted at most once until it is replaced on disk.
//!
//! Every field is best-effort: when a platform API is unavailable the identity
//! degrades to a real "unknown" / "unsigned" / "unpackaged" verdict rather than
//! a fabricated value.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

/// Resolved code identity for a binary. Fields are platform-specific and
/// best-effort; anything unavailable is `None` (never fabricated).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Identity {
    /// macOS code-signing Team Identifier (e.g. `EQHXZ8M8AV`), if signed.
    pub team_id: Option<String>,
    /// macOS signing-authority verdict (e.g. `Apple System`, `Apple-issued`).
    pub authority: Option<String>,
    /// Linux owning package (dpkg/rpm), if the file is tracked by a package.
    pub package: Option<String>,
    /// SHA-256 of the on-disk executable image (Linux), lowercase hex.
    pub sha256: Option<String>,
    /// Short rollup label used to group flows by publisher in the UI.
    pub label: String,
}

impl Identity {
    /// The identity used when the executable path or its signature cannot be
    /// read. A real verdict, not a placeholder pretending to be resolved.
    pub fn unknown() -> Self {
        Identity {
            label: "unknown".to_string(),
            ..Default::default()
        }
    }
}

/// Cache key: the on-disk executable's `(device, inode, mtime)`. When any of
/// the three changes the binary has been replaced and must be re-fingerprinted.
type CacheKey = (u64, u64, i64);

fn identity_cache() -> &'static Mutex<HashMap<CacheKey, Identity>> {
    static CACHE: OnceLock<Mutex<HashMap<CacheKey, Identity>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Look the identity up in the `(dev,inode,mtime)` cache, resolving via `f`
/// exactly once per distinct on-disk binary. `f` is NOT called on a cache hit.
fn cache_lookup_or_insert(key: CacheKey, f: impl FnOnce() -> Identity) -> Identity {
    {
        let cache = identity_cache().lock().unwrap_or_else(|e| e.into_inner());
        if let Some(id) = cache.get(&key) {
            return id.clone();
        }
    }
    // Resolve outside the lock (signing / package queries can be slow), then
    // publish. A concurrent resolver may have won the race — keep whichever
    // landed first; both describe the same on-disk bytes.
    let resolved = f();
    let mut cache = identity_cache().lock().unwrap_or_else(|e| e.into_inner());
    cache.entry(key).or_insert_with(|| resolved.clone()).clone()
}

/// Resolve the code identity for a live PID. Returns an "unknown" identity if
/// the executable path or its metadata cannot be read.
pub fn identity_for(pid: u32) -> Identity {
    match crate::util::procinfo::exe_path_for(pid) {
        Some(exe) => identity_for_path(&exe),
        None => Identity::unknown(),
    }
}

/// Resolve (and cache) the code identity for an on-disk executable path.
pub fn identity_for_path(exe: &Path) -> Identity {
    use std::os::unix::fs::MetadataExt;
    let meta = match std::fs::metadata(exe) {
        Ok(m) => m,
        Err(_) => return Identity::unknown(),
    };
    let key = (meta.dev(), meta.ino(), meta.mtime());
    let exe = exe.to_path_buf();
    cache_lookup_or_insert(key, move || resolve(&exe))
}

#[cfg(target_os = "macos")]
fn resolve(exe: &Path) -> Identity {
    macos::resolve(exe)
}

#[cfg(target_os = "linux")]
fn resolve(exe: &Path) -> Identity {
    linux::resolve(exe)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn resolve(_exe: &Path) -> Identity {
    Identity::unknown()
}

// ─── macOS: code-signing identity ─────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod macos {
    use super::Identity;
    use std::ffi::c_void;
    use std::path::Path;
    use std::ptr;
    use std::str::FromStr;

    use core_foundation::base::TCFType;
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::string::CFString;
    use core_foundation::url::CFURL;
    use core_foundation_sys::base::OSStatus;
    use core_foundation_sys::dictionary::{CFDictionaryGetValue, CFDictionaryRef};
    use core_foundation_sys::string::CFStringRef;
    use security_framework::os::macos::code_signing::{Flags, SecRequirement, SecStaticCode};

    // Signing-information selector for SecCodeCopySigningInformation. Neither
    // `security-framework` nor its `-sys` crate expose that call or its info
    // keys, so we bind them directly. `kSecCSSigningInformation = 1 << 1`
    // (CSCommon.h). Linking `Security` here is idempotent with the -sys crate.
    const K_SEC_CS_SIGNING_INFORMATION: u32 = 1 << 1;

    // errSecCSUnsigned: the code object carries no signature at all.
    const ERR_SEC_CS_UNSIGNED: i32 = -67062;

    #[link(name = "Security", kind = "framework")]
    unsafe extern "C" {
        fn SecCodeCopySigningInformation(
            code: *const c_void,
            flags: u32,
            information: *mut CFDictionaryRef,
        ) -> OSStatus;

        static kSecCodeInfoTeamIdentifier: CFStringRef;
    }

    pub fn resolve(exe: &Path) -> Identity {
        let url = match CFURL::from_path(exe, false) {
            Some(u) => u,
            None => return Identity::unknown(),
        };
        let code = match SecStaticCode::from_path(&url, Flags::NONE) {
            Ok(c) => c,
            Err(_) => return Identity::unknown(),
        };

        let team_id = copy_team_identifier(&code);
        let authority = signing_authority(&code);

        // Rollup label: prefer the durable Team ID (the vendor identity), then
        // fall back to the authority verdict, then to the unsigned verdict.
        let label = if let Some(ref t) = team_id {
            t.clone()
        } else {
            match authority.as_deref() {
                Some(a) if a.starts_with("Apple") => "apple".to_string(),
                Some("Ad-hoc") => "adhoc-signed".to_string(),
                Some(_) => "signed".to_string(),
                None => "unsigned-binary".to_string(),
            }
        };

        Identity {
            team_id,
            authority,
            package: None,
            sha256: None,
            label,
        }
    }

    /// Read the Team Identifier out of the code's signing information
    /// dictionary. `None` for Apple-system binaries (no team) and unsigned code.
    fn copy_team_identifier(code: &SecStaticCode) -> Option<String> {
        let mut info: CFDictionaryRef = ptr::null();
        let status = unsafe {
            SecCodeCopySigningInformation(
                code.as_concrete_TypeRef() as *const c_void,
                K_SEC_CS_SIGNING_INFORMATION,
                &mut info,
            )
        };
        if status != 0 || info.is_null() {
            return None;
        }
        // Take ownership of the returned (+1) dictionary reference.
        let dict = unsafe {
            CFDictionary::<*const c_void, *const c_void>::wrap_under_create_rule(info)
        };
        let value = unsafe {
            CFDictionaryGetValue(
                dict.as_concrete_TypeRef(),
                kSecCodeInfoTeamIdentifier as *const c_void,
            )
        };
        if value.is_null() {
            return None;
        }
        let s = unsafe { CFString::wrap_under_get_rule(value as CFStringRef) }.to_string();
        if s.is_empty() { None } else { Some(s) }
    }

    /// Classify the signing authority via code-requirement probes. Returns a
    /// verdict string, or `None` when the binary is unsigned.
    fn signing_authority(code: &SecStaticCode) -> Option<String> {
        for (req, label) in [
            ("anchor apple", "Apple System"),
            ("anchor apple generic", "Apple-issued"),
        ] {
            if let Ok(requirement) = SecRequirement::from_str(req)
                && code.check_validity(Flags::NONE, &requirement).is_ok()
            {
                return Some(label.to_string());
            }
        }
        // Not Apple-anchored. Distinguish an ad-hoc / third-party signature from
        // an entirely unsigned binary: a `trusted` probe fails one of two ways.
        match SecRequirement::from_str("anchor trusted") {
            Ok(req) => match code.check_validity(Flags::NONE, &req) {
                Ok(()) => Some("Trusted".to_string()),
                Err(e) => {
                    if e.code() == ERR_SEC_CS_UNSIGNED {
                        None
                    } else {
                        Some("Ad-hoc".to_string())
                    }
                }
            },
            Err(_) => None,
        }
    }
}

// ─── Linux: owning package + executable SHA-256 ───────────────────────────────

#[cfg(target_os = "linux")]
mod linux {
    use super::Identity;
    use std::path::Path;
    use std::process::Command;

    pub fn resolve(exe: &Path) -> Identity {
        let package = owning_package(exe);
        let sha256 = std::fs::read(exe)
            .ok()
            .map(|bytes| super::sha256_hex(&bytes));
        let label = package
            .clone()
            .unwrap_or_else(|| "unpackaged-binary".to_string());
        Identity {
            team_id: None,
            authority: None,
            package,
            sha256,
            label,
        }
    }

    /// Query the distro package that owns `exe`. dpkg first (Debian/Ubuntu),
    /// then rpm (Fedora/RHEL/SUSE). `None` when neither tool tracks the file.
    fn owning_package(exe: &Path) -> Option<String> {
        // dpkg -S <path> → "pkg:arch: /path" or "pkg: /path"
        if let Ok(out) = Command::new("dpkg").arg("-S").arg(exe).output()
            && out.status.success()
            && let Some(line) = String::from_utf8_lossy(&out.stdout).lines().next()
            && let Some((pkg, _)) = line.split_once(':')
        {
            let pkg = pkg.trim();
            if !pkg.is_empty() {
                return Some(pkg.to_string());
            }
        }
        // rpm -qf <path> → package NEVRA (or "... is not owned by any package")
        if let Ok(out) = Command::new("rpm").arg("-qf").arg(exe).output()
            && out.status.success()
        {
            let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !name.is_empty() && !name.contains("not owned by") {
                return Some(name);
            }
        }
        None
    }
}

/// FIPS 180-4 SHA-256 of a byte slice, returned as lowercase hex. Fingerprints
/// the executable image on Linux without pulling in a hashing crate.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in w.iter_mut().enumerate().take(16) {
            let b = i * 4;
            *word = u32::from_be_bytes([chunk[b], chunk[b + 1], chunk[b + 2], chunk[b + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (dst, v) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *dst = dst.wrapping_add(v);
        }
    }

    let mut out = String::with_capacity(64);
    for word in h {
        out.push_str(&format!("{:08x}", word));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_empty_string_known_vector() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_abc_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sha256_multi_block_known_vector() {
        // 56 bytes forces a second padded block (message-length overflow of
        // the first 64-byte block's usable 56 bytes).
        let input = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        assert_eq!(
            sha256_hex(input),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
    }

    #[test]
    fn cache_resolves_once_per_key_and_hits_thereafter() {
        // A key that no real (dev,inode,mtime) will collide with, so the global
        // cache stays deterministic across the suite.
        let key = (u64::MAX, u64::MAX, 4242);

        let first = cache_lookup_or_insert(key, || Identity {
            label: "vendor-x".to_string(),
            team_id: Some("TEAMX".to_string()),
            ..Default::default()
        });
        assert_eq!(first.label, "vendor-x");
        assert_eq!(first.team_id.as_deref(), Some("TEAMX"));

        // Second call MUST hit the cache — the closure would panic if invoked.
        let second = cache_lookup_or_insert(key, || panic!("cache miss: resolver re-ran"));
        assert_eq!(first, second);
    }

    #[test]
    fn identity_for_unknown_pid_degrades_to_unknown() {
        // No process owns u32::MAX, so there is no exe path to fingerprint.
        let id = identity_for(u32::MAX);
        assert_eq!(id.label, "unknown");
        assert!(id.team_id.is_none());
        assert!(id.package.is_none());
    }

    #[test]
    fn unknown_identity_has_no_resolved_fields() {
        let id = Identity::unknown();
        assert_eq!(id.label, "unknown");
        assert!(id.team_id.is_none());
        assert!(id.authority.is_none());
        assert!(id.package.is_none());
        assert!(id.sha256.is_none());
    }
}

