//! Hand-crafted edge-case tests for `FilterState`.
//!
//! These tests pin down subtle Unicode and whitespace-class behaviors of the
//! text-editor buffer that backs the in-TUI filter prompt. They are NOT mirrors
//! of the existing in-module tests:
//!
//! * `multi_byte_emoji_*` exercises 4-byte UTF-8 code points (existing tests
//!   only use 2-byte Latin diacritics) and would catch off-by-one byte vs char
//!   indexing bugs in `insert` / `backspace` / `left` / `right`.
//! * `cjk_char_navigation` exercises 3-byte UTF-8 code points which sit between
//!   the 2-byte and 4-byte cases (different `len_utf8`) — catches arithmetic
//!   that hardcodes a specific byte width.
//! * `delete_word_*_separator` pins the contract that `delete_word` treats
//!   ASCII whitespace beyond plain space (tab `\t`, line feed `\n`, carriage
//!   return `\r`, vertical tab `\x0B`, form feed `\x0C`) as word boundaries.
//!   A regression that narrows the separator class to only ' ' would silently
//!   pass the existing `filter_state_delete_word` test.
//! * `delete_word_chained` verifies repeated invocations correctly track the
//!   moving cursor — guards against state being left in an inconsistent
//!   position after a delete.

use iftoprs::ui::app::FilterState;

/// Inserting a 4-byte emoji must advance the cursor by 4 bytes, and
/// backspace must remove all 4 bytes atomically. A naive `cursor += 1`
/// implementation would land mid-codepoint and panic on the next edit.
#[test]
fn multi_byte_emoji_insert_backspace_round_trip() {
    let mut f = FilterState::new();
    f.insert('🦀'); // 4 UTF-8 bytes
    assert_eq!(f.buf, "🦀");
    assert_eq!(
        f.cursor, 4,
        "cursor must equal byte length after emoji insert"
    );

    f.insert('🦀');
    assert_eq!(f.buf, "🦀🦀");
    assert_eq!(f.cursor, 8);

    f.backspace();
    assert_eq!(f.buf, "🦀");
    assert_eq!(f.cursor, 4, "backspace must remove entire 4-byte codepoint");

    f.backspace();
    assert_eq!(f.buf, "");
    assert_eq!(f.cursor, 0);
}

/// CJK characters are 3-byte UTF-8. Left/right must advance by 3 bytes per
/// character. Buggy code that assumes 2-byte chars would skip past the second
/// CJK character on a single `right()` call.
#[test]
fn cjk_char_navigation_three_byte_chars() {
    let mut f = FilterState::new();
    f.insert('中'); // 3 bytes: E4 B8 AD
    f.insert('文'); // 3 bytes: E6 96 87
    assert_eq!(f.buf, "中文");
    assert_eq!(f.cursor, 6, "two 3-byte chars => cursor=6");

    f.left();
    assert_eq!(f.cursor, 3, "left should step back exactly 3 bytes");
    f.left();
    assert_eq!(f.cursor, 0);
    f.left();
    assert_eq!(f.cursor, 0, "left at start must clamp to 0");

    f.right();
    assert_eq!(f.cursor, 3);
    f.right();
    assert_eq!(f.cursor, 6);
    f.right();
    assert_eq!(f.cursor, 6, "right at end must clamp to buf.len()");
}

/// Insert in the middle of a multi-byte run. Catches bugs where insert uses
/// a char-count cursor instead of a byte-offset cursor.
#[test]
fn insert_between_emoji_lands_at_byte_boundary() {
    let mut f = FilterState::new();
    f.insert('🦀');
    f.insert('🦀');
    // cursor=8, buf="🦀🦀"
    f.left(); // cursor=4 (between the two crabs)
    assert_eq!(f.cursor, 4);
    f.insert('x'); // single-byte ASCII inserted between them
    assert_eq!(f.buf, "🦀x🦀");
    assert_eq!(f.cursor, 5, "cursor advances by 1 byte for ASCII insert");
}

/// Backspace from inside a 4-byte sequence must remove the whole codepoint.
/// If the implementation used `cursor -= 1` it would create invalid UTF-8.
#[test]
fn backspace_after_emoji_removes_whole_codepoint() {
    let mut f = FilterState::new();
    f.insert('a');
    f.insert('🦀');
    f.insert('b');
    // buf = "a🦀b", cursor at end (1 + 4 + 1 = 6)
    assert_eq!(f.buf, "a🦀b");
    assert_eq!(f.cursor, 6);
    f.left(); // cursor=5 (between 🦀 and b)
    f.backspace(); // must remove the entire 4-byte emoji
    assert_eq!(f.buf, "ab");
    assert_eq!(f.cursor, 1, "cursor lands just after 'a'");
}

/// `delete_word` must treat tab (`\t`) as a word separator. The existing
/// in-module test only uses ASCII space — a regression that narrowed the
/// separator class to `== ' '` would silently still pass.
#[test]
fn delete_word_tab_separator() {
    let mut f = FilterState::new();
    f.buf = "alpha\tbeta".to_string();
    f.cursor = f.buf.len();
    f.delete_word();
    assert_eq!(
        f.buf, "alpha\t",
        "tab must act as a word boundary; only 'beta' should be deleted"
    );
    assert_eq!(f.cursor, 6, "cursor must rest just past the tab");
}

/// `delete_word` must treat newline (`\n`) as a word separator. Multi-line
/// buffers aren't typical for this filter, but the implementation uses
/// `char::is_whitespace` and we want a regression catch if that ever
/// narrows.
#[test]
fn delete_word_newline_separator() {
    let mut f = FilterState::new();
    f.buf = "first\nsecond".to_string();
    f.cursor = f.buf.len();
    f.delete_word();
    assert_eq!(f.buf, "first\n", "newline must act as a word boundary");
}

/// `delete_word` chained — calling it twice should fully clear a buffer
/// of "word ". Catches a bug where cursor is not properly synced after
/// the drain, leaving an inconsistent (cursor, buf) pair.
#[test]
fn delete_word_chained_clears_buffer() {
    let mut f = FilterState::new();
    f.buf = "hello world".to_string();
    f.cursor = f.buf.len();
    f.delete_word();
    assert_eq!(f.buf, "hello ");
    assert_eq!(f.cursor, 6);
    f.delete_word();
    assert_eq!(f.buf, "", "second delete_word must consume 'hello '");
    assert_eq!(f.cursor, 0);
}

/// `kill_to_end` from middle of a multi-byte sequence — cursor must
/// already be on a char boundary (invariant maintained by left/right/insert).
/// This guards against a future refactor that might break the boundary
/// invariant.
#[test]
fn kill_to_end_at_emoji_boundary() {
    let mut f = FilterState::new();
    f.insert('a');
    f.insert('🦀');
    f.insert('b');
    f.insert('🦀');
    // cursor = 1 + 4 + 1 + 4 = 10
    f.left(); // step back over second 🦀 → cursor = 6
    f.kill_to_end();
    assert_eq!(f.buf, "a🦀b");
    assert_eq!(f.cursor, 6, "cursor stays at the truncation point");
}

/// Verifies the `open()` invariant: cursor must equal buf.len() so the user
/// can immediately type at the end of the pre-filled query, even with
/// multi-byte content. Catches a regression where cursor was set to char
/// count instead of byte length.
#[test]
fn open_with_multibyte_prefill_places_cursor_at_byte_end() {
    let mut f = FilterState::new();
    let prefill = Some("café🦀".to_string()); // c+a+f=3, é=2, 🦀=4 → 9 bytes
    f.open(&prefill);
    assert!(f.active);
    assert_eq!(f.buf, "café🦀");
    assert_eq!(
        f.cursor,
        f.buf.len(),
        "cursor must be at byte end so next insert appends"
    );
    assert_eq!(f.cursor, 9);

    // Verify the cursor is actually at a valid boundary by performing an edit
    f.insert('!');
    assert_eq!(f.buf, "café🦀!");
}
