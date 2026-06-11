//! Regression tests for `FilterState::delete_word` against MULTI-BYTE
//! whitespace separators.
//!
//! `src/ui/app.rs` documents (in the `delete_word` body comment) that the
//! original implementation used `rfind(char::is_whitespace).map(|i| i + 1)`,
//! which assumed the matched whitespace was a single byte. That panicked with
//! an `is_char_boundary` assertion when the separator was a multi-byte UTF-8
//! whitespace character such as NBSP (U+00A0, 2 bytes) or the CJK ideographic
//! space (U+3000, 3 bytes), because `word_start = i + 1` landed in the middle
//! of the separator's UTF-8 sequence and the subsequent `String::drain` sliced
//! on a non-char boundary.
//!
//! The existing `tests/filter_state_unicode_edge.rs` only exercises ASCII
//! whitespace separators (`\t`, `\n`) — every ASCII whitespace char is 1 byte,
//! so those tests would STILL PASS against the buggy `i + 1` arithmetic. The
//! tests below are the only ones that pin the multi-byte-separator contract:
//! they fail (panic) under the old `i + 1` logic and pass under the current
//! `i + c.len_utf8()` logic. Not a mirror — different bug class (separator
//! width, not deleted-word content width).

use iftoprs::ui::app::FilterState;

/// NBSP (U+00A0) is a 2-byte whitespace char. `delete_word` must compute the
/// word start as `nbsp_offset + 2`, not `nbsp_offset + 1`. The naive `+ 1`
/// would drain from inside the NBSP sequence and panic on a non-char boundary.
#[test]
fn delete_word_nbsp_separator_two_byte() {
    let mut f = FilterState::new();
    // "alpha" + NBSP + "beta": 5 + 2 + 4 = 11 bytes.
    f.buf = "alpha\u{00A0}beta".to_string();
    f.cursor = f.buf.len();
    assert_eq!(f.cursor, 11, "precondition: NBSP is 2 bytes");

    f.delete_word();

    assert_eq!(
        f.buf, "alpha\u{00A0}",
        "only 'beta' should be removed; the 2-byte NBSP separator stays"
    );
    assert_eq!(
        f.cursor, 7,
        "cursor must rest just past the 2-byte NBSP (5 + 2), on a char boundary"
    );
    // Buffer must still be valid UTF-8 and editable — proves the drain did not
    // corrupt the byte sequence.
    f.insert('!');
    assert_eq!(f.buf, "alpha\u{00A0}!");
}

/// CJK ideographic space (U+3000) is a 3-byte whitespace char. This sits
/// between the 2-byte NBSP case and any 1-byte ASCII case, so it catches
/// arithmetic that hardcodes a 1- or 2-byte separator width.
#[test]
fn delete_word_ideographic_space_separator_three_byte() {
    let mut f = FilterState::new();
    // "host" + U+3000 + "port": 4 + 3 + 4 = 11 bytes.
    f.buf = "host\u{3000}port".to_string();
    f.cursor = f.buf.len();
    assert_eq!(f.cursor, 11, "precondition: U+3000 is 3 bytes");

    f.delete_word();

    assert_eq!(
        f.buf, "host\u{3000}",
        "only 'port' removed; the 3-byte ideographic space separator stays"
    );
    assert_eq!(
        f.cursor, 7,
        "cursor must rest just past the 3-byte separator (4 + 3)"
    );
    f.insert('x');
    assert_eq!(f.buf, "host\u{3000}x");
}

/// `delete_word` with the cursor sitting after TRAILING multi-byte whitespace:
/// the `trim_end()` step must skip the multi-byte trailing whitespace, then the
/// preceding word is deleted. Catches a bug where `trim_end` interplay with the
/// multi-byte offset math leaves a dangling partial codepoint.
#[test]
fn delete_word_trailing_nbsp_then_word_deleted() {
    let mut f = FilterState::new();
    // "tcp" + NBSP (trailing): delete_word must trim the NBSP and remove "tcp".
    f.buf = "tcp\u{00A0}".to_string();
    f.cursor = f.buf.len();
    assert_eq!(f.cursor, 5, "precondition: 'tcp' (3) + NBSP (2) = 5 bytes");

    f.delete_word();

    assert_eq!(
        f.buf, "",
        "trailing NBSP is trimmed, then 'tcp' deleted — buffer empties"
    );
    assert_eq!(f.cursor, 0);
}
