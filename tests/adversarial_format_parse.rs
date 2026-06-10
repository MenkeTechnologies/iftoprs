//! Adversarial tests for `iftoprs::util::format` and `iftoprs::capture::parser`
//! aimed at specific bug classes that aren't covered by the existing
//! comprehensive value-table tests.
//!
//! Each test is hand-crafted around a concrete failure mode — none would be
//! caught by the boundary-table tests in `src/util/format.rs` because they
//! probe invariants over the *shape* of the output, not specific numeric
//! values.

use std::net::IpAddr;

use iftoprs::capture::parser;
use iftoprs::data::flow::Direction;
use iftoprs::util::format::{readable_size, sparkline};

// ─── sparkline Unicode width invariant ───────────────────────────────
//
// `sparkline` is consumed by TUI renderers (`ratatui`) which assume each
// glyph occupies exactly one terminal column. The block-element glyphs
// U+2581..U+2588 plus ASCII space are the only legal outputs. Each block
// glyph encodes as exactly 3 bytes in UTF-8.
//
// If a future refactor accidentally swaps in a 2- or 4-byte glyph (e.g. an
// emoji or combining mark), `String::len()` no longer matches column count,
// which silently corrupts the TUI alignment in `render.rs`. This test pins
// the per-char byte length so any such regression is caught at the unit
// boundary — long before the user sees a broken table.

#[test]
fn sparkline_only_emits_one_column_glyphs() {
    // Mix of zero, mid, and max values to hit every BLOCKS branch.
    let data: Vec<u64> = (0u64..=8).collect();
    let out = sparkline(&data, data.len());

    // Every emitted char must be either the space placeholder or one of the
    // eight U+2581..U+2588 block-element glyphs.
    for ch in out.chars() {
        let ok = ch == ' ' || ('\u{2581}'..='\u{2588}').contains(&ch);
        assert!(
            ok,
            "sparkline emitted illegal glyph {:?} (U+{:04X})",
            ch, ch as u32
        );
    }

    // ASCII space is 1 byte, each block glyph is 3 bytes. Verify the total
    // byte length matches that exact breakdown — catches any silent swap to
    // a different-width glyph.
    let space_count = out.chars().filter(|&c| c == ' ').count();
    let block_count = out.chars().filter(|&c| c != ' ').count();
    assert_eq!(
        out.len(),
        space_count + 3 * block_count,
        "sparkline byte length disagrees with 1-byte space + 3-byte block accounting: {:?}",
        out
    );
}

// ─── sparkline rounding does not panic on u64::MAX with tiny values ──
//
// Internal computation does `(v as f64 / max as f64) * 7.0`. When `max =
// u64::MAX`, the conversion loses precision and many small `v` values get
// the same float ratio. The bug class here is `(...).round() as usize`
// wrapping when the rounded value is outside `[0, usize::MAX]`. With f64 ×
// 7.0 capped at 7.0 by the (v <= max) invariant, `round()` returns
// 0.0..=7.0, all in-range. But a future change that uses `* 8.0` instead
// of `* 7.0` plus `.min(7)` post-clamp would let `round()` return 8.0,
// then `idx.min(7)` saves the day — but only because of that explicit
// `.min(7)`. This test pins that defensive clamp.

#[test]
fn sparkline_with_u64_max_top_block_never_panics_or_misindexes() {
    // Maximum representable u64 value as both `v` and `max`.
    let out = sparkline(&[u64::MAX], 1);
    // Singleton non-zero must render the tallest block.
    assert_eq!(out, "█", "expected single tallest block for [u64::MAX]");

    // Asymmetric: include something tiny next to u64::MAX — the small entry
    // must NOT panic the as-usize cast and must stay below the top block.
    let out = sparkline(&[1, u64::MAX], 2);
    let chars: Vec<char> = out.chars().collect();
    assert_eq!(chars.len(), 2);
    // 1 / u64::MAX * 7.0 underflows to ~0.0 → idx = 0 → ▁
    assert_eq!(
        chars[0], '\u{2581}',
        "tiny value beside u64::MAX should render bottom block"
    );
    assert_eq!(chars[1], '\u{2588}', "u64::MAX should render top block");
}

// ─── readable_size: non-finite inputs must not crash ─────────────────
//
// Bandwidth rates come from f64 arithmetic over packet counters. If the
// flow tracker ever divides by an instantaneous interval of zero (e.g.
// during a slot rotation hiccup), the resulting rate could be ±inf or
// NaN. The format helper has no explicit guard. This test pins the
// current behavior — it must NOT panic — and surfaces the actual rendered
// string. A future change that swaps `<` for `>=` in the branches or
// reorders units would shift NaN classification silently; the
// pinning here ensures any such reordering is caught.

#[test]
fn readable_size_does_not_panic_on_nonfinite_inputs() {
    // The cascade of `if value < 1_000.0` comparisons all yield `false`
    // for NaN, so NaN falls through to the terabyte branch. The exact
    // rendered text is `NaNTB` / `NaNTb`. We pin this so a refactor that
    // accidentally swaps the comparison direction (or introduces a panic
    // path on non-finite) is caught.
    let nan_bytes = readable_size(f64::NAN, true);
    assert!(
        nan_bytes.contains("NaN"),
        "expected NaN tag in output: {:?}",
        nan_bytes
    );
    assert!(
        nan_bytes.ends_with("TB"),
        "NaN should fall through to TB branch: {:?}",
        nan_bytes
    );

    let nan_bits = readable_size(f64::NAN, false);
    assert!(
        nan_bits.contains("NaN"),
        "expected NaN tag in bit output: {:?}",
        nan_bits
    );
    assert!(
        nan_bits.ends_with("Tb"),
        "NaN should fall through to Tb branch: {:?}",
        nan_bits
    );

    // +Infinity is greater than every threshold; same TB branch.
    let pos_inf = readable_size(f64::INFINITY, true);
    assert!(
        pos_inf.contains("inf") || pos_inf.contains("Inf") || pos_inf.contains("INF"),
        "expected inf in output: {:?}",
        pos_inf
    );
    assert!(pos_inf.ends_with("TB"));

    // -Infinity is less than every positive threshold; falls into the
    // first `< 1_000.0` branch (uses `{:.0}{}`). Must not panic.
    let neg_inf = readable_size(f64::NEG_INFINITY, true);
    assert!(!neg_inf.is_empty(), "neg-inf should not render empty");
}

// ─── parser: zero-length TCP/UDP segment must yield zero ports, not crash
//
// In `parse_ports` the protocol-match arm checks `data.len() >= header_len
// + 4`. If header_len computed from the IPv4 IHL field were ever wrong
// (e.g. someone changed the `* 4` to `* 8`), the resulting indexing in the
// `(src, dst)` arm could read past the slice. The test below builds a
// minimum-legal IPv4 TCP packet with NO room for the port bytes (total
// length == IHL header length exactly) and asserts both ports come out as
// zero — proving the `>=` guard, not the indexing, is what decides the
// branch.

#[test]
fn parse_raw_ipv4_tcp_with_no_room_for_ports_yields_zero_ports_no_panic() {
    // IHL=5 → 20-byte header. Slice is exactly 20 bytes — no port bytes.
    let mut raw = vec![0u8; 20];
    raw[0] = 0x45; // version 4, IHL 5
    raw[2] = 0;
    raw[3] = 20; // total_len = header only
    raw[9] = 6; // TCP
    raw[12..16].copy_from_slice(&[10, 0, 0, 1]);
    raw[16..20].copy_from_slice(&[10, 0, 0, 2]);

    let result = parser::parse_raw(&raw, None).expect("legal IPv4 header alone should still parse");
    assert_eq!(
        result.key.src_port, 0,
        "no port bytes available → src_port must be 0"
    );
    assert_eq!(
        result.key.dst_port, 0,
        "no port bytes available → dst_port must be 0"
    );
    assert_eq!(result.len, 20);
}

// ─── parser: IHL=15 (max) consumes 60 bytes and STILL parses ports if room
//
// IHL field is 4 bits (max value 15 → 60-byte header). A packet with
// IHL=15 and exactly 60+4 bytes of TCP payload must read ports starting
// at offset 60. If anyone replaces `* 4` with a hard-coded `20`, this
// test breaks because the ports are read from the wrong offset.

#[test]
fn parse_raw_ipv4_ihl_max_reads_ports_at_offset_sixty() {
    let mut raw = vec![0u8; 60 + 4];
    raw[0] = 0x4F; // version 4, IHL 15 (max)
    raw[2] = 0;
    raw[3] = 64; // total_len = 60 hdr + 4 ports
    raw[9] = 17; // UDP
    raw[12..16].copy_from_slice(&[10, 0, 0, 1]);
    raw[16..20].copy_from_slice(&[10, 0, 0, 2]);
    // ports begin at offset 60
    raw[60] = 0xAB;
    raw[61] = 0xCD; // src port 0xABCD = 43981
    raw[62] = 0x12;
    raw[63] = 0x34; // dst port 0x1234 = 4660

    let result = parser::parse_raw(&raw, None).expect("IHL=15 packet should parse");
    // Ports are normalized — the lower one becomes src_port.
    let (lo, hi) = if result.key.src_port < result.key.dst_port {
        (result.key.src_port, result.key.dst_port)
    } else {
        (result.key.dst_port, result.key.src_port)
    };
    assert_eq!(lo, 0x1234, "lower port should be 0x1234");
    assert_eq!(hi, 0xABCD, "higher port should be 0xABCD");
}

// ─── parser direction symmetry sanity (orthogonal to existing tests) ─
//
// Existing `both_directions_same_canonical_key` asserts the canonical key
// matches across direction reversals on Ethernet/IPv4. This test extends
// the invariant to a different transport (UDP) and a different ethertype
// (IPv6) — catching any branch where IPv6 normalize() is hooked up
// differently from IPv4 in `parse_ipv6`.

#[test]
fn parse_ethernet_ipv6_udp_direction_reversal_produces_same_canonical_key() {
    // Helper to build an Ethernet/IPv6/UDP frame: 14 eth + 40 ipv6 + 8 udp.
    fn make(src: [u8; 16], dst: [u8; 16], src_port: u16, dst_port: u16) -> Vec<u8> {
        let mut pkt = vec![0u8; 14 + 48];
        pkt[12] = 0x86;
        pkt[13] = 0xDD; // IPv6
        pkt[14] = 0x60; // version 6
        pkt[18] = 0;
        pkt[19] = 8; // payload length = 8 (UDP header)
        pkt[20] = 17; // next header = UDP
        pkt[21] = 64;
        pkt[22..38].copy_from_slice(&src);
        pkt[38..54].copy_from_slice(&dst);
        pkt[54..56].copy_from_slice(&src_port.to_be_bytes());
        pkt[56..58].copy_from_slice(&dst_port.to_be_bytes());
        pkt
    }

    let mut a_ip = [0u8; 16];
    a_ip[0] = 0x20;
    a_ip[1] = 0x01;
    a_ip[15] = 0x01; // 2001::1
    let mut b_ip = [0u8; 16];
    b_ip[0] = 0x20;
    b_ip[1] = 0x01;
    b_ip[15] = 0x02; // 2001::2

    let forward = make(a_ip, b_ip, 5000, 53);
    let reverse = make(b_ip, a_ip, 53, 5000);

    let local: IpAddr = "2001::".parse().unwrap();
    let f = parser::parse_ethernet(&forward, Some((local, 16))).unwrap();
    let r = parser::parse_ethernet(&reverse, Some((local, 16))).unwrap();

    assert_eq!(
        f.key, r.key,
        "canonical IPv6/UDP key must match under direction reversal"
    );
    // Both endpoints sit inside the local /16, so `determine_direction` returns
    // `Sent` for both raw orientations. Normalization then swaps the reverse
    // packet (2001::2 > 2001::1), which inverts its direction. The contract:
    // wire-reversed packets land on the SAME canonical key but in OPPOSITE
    // directions, mirroring the IPv4 invariant already pinned by
    // `both_directions_same_canonical_key` for ETH/IPv4.
    assert_ne!(
        f.direction, r.direction,
        "wire reversal must produce opposite directions even when both endpoints are local"
    );
    assert_eq!(f.direction, Direction::Sent);
    assert_eq!(r.direction, Direction::Received);
}
