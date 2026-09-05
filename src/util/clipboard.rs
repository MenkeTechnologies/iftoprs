//! Clipboard writes for the TUI's copy key.
//!
//! Every helper binary is optional: a stripped `PATH`, a headless box with no
//! X/Wayland tool, or an ssh session all leave the process with nothing to
//! spawn. The chain below tries each candidate in turn and falls back to the
//! OSC 52 terminal escape, which the terminal emulator itself handles.

use std::io;

/// External clipboard helpers, tried in order; the first one that exists wins.
/// `pbcopy` is also tried by absolute path so a stripped `PATH` (sudo, launchd)
/// still copies.
pub const CLIPBOARD_CMDS: &[(&str, &[&str])] = &[
    ("pbcopy", &[]),
    ("/usr/bin/pbcopy", &[]),
    ("wl-copy", &[]),
    ("xclip", &["-selection", "clipboard"]),
    ("xsel", &["--clipboard", "--input"]),
    ("clip.exe", &[]),
    ("termux-clipboard-set", &[]),
];

/// Copy `text` to the system clipboard, returning the mechanism that took it.
pub fn copy_to_clipboard(text: &str) -> Result<String, String> {
    let mut last_err = String::new();
    for (cmd, args) in CLIPBOARD_CMDS {
        match run_clipboard_cmd(cmd, args, text) {
            Ok(()) => return Ok((*cmd).to_string()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => last_err = format!("{}: {}", cmd, e),
        }
    }
    match copy_via_osc52(text) {
        Ok(()) => Ok("osc52".to_string()),
        Err(e) if last_err.is_empty() => Err(format!("no clipboard helper found; osc52: {}", e)),
        Err(_) => Err(last_err),
    }
}

/// Pipe `text` into a clipboard helper and wait for it to finish.
pub fn run_clipboard_cmd(cmd: &str, args: &[&str], text: &str) -> io::Result<()> {
    use std::io::Write;
    let mut child = std::process::Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    // Take the pipe so it closes before the wait — helpers read until EOF.
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("clipboard helper has no stdin"))?;
    let written = stdin.write_all(text.as_bytes());
    drop(stdin);
    let status = child.wait()?;
    if !status.success() {
        return Err(io::Error::other(format!("exited with {}", status)));
    }
    // A helper that exits 0 without draining its stdin leaves the write with
    // EPIPE, and whether the write or the exit wins the race decides whether it
    // is seen at all — the same call reported success on one runner and
    // "Broken pipe" on the next. The exit status is the helper's own answer
    // about whether it took the text, so a broken pipe from one that succeeded
    // is not an error to report over it. Every other write failure still is.
    match written {
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        other => other,
    }
}

/// Build the OSC 52 clipboard escape, wrapped in the tmux passthrough sequence
/// (escapes doubled) when running inside tmux.
pub fn osc52_sequence(text: &str, tmux: bool) -> String {
    let payload = format!("\x1b]52;c;{}\x07", base64_encode(text.as_bytes()));
    if tmux {
        format!("\x1bPtmux;{}\x1b\\", payload.replace('\x1b', "\x1b\x1b"))
    } else {
        payload
    }
}

/// Hand the text to the terminal itself via OSC 52 — the only path that works
/// with no helper binary, including over ssh.
pub fn copy_via_osc52(text: &str) -> io::Result<()> {
    use std::io::Write;
    let seq = osc52_sequence(text, std::env::var_os("TMUX").is_some());
    let mut out = io::stdout();
    out.write_all(seq.as_bytes())?;
    out.flush()
}

/// Encode `data` as standard base64 (RFC 4648, with `=` padding).
#[must_use]
pub fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = chunk.get(1).map_or(0, |b| u32::from(*b));
        let b2 = chunk.get(2).map_or(0, |b| u32::from(*b));
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[(n >> 18) as usize & 0x3f] as char);
        out.push(ALPHABET[(n >> 12) as usize & 0x3f] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 0x3f] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 0x3f] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_encode_matches_rfc4648_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64_encode(&[0xff, 0xfe, 0xfd]), "//79");
    }

    /// A missing helper must surface as `NotFound` so the chain moves on to the
    /// next candidate instead of reporting "Copy failed: os error 2".
    #[test]
    fn missing_clipboard_helper_reports_not_found() {
        let err = run_clipboard_cmd("iftoprs-no-such-clipboard-helper", &[], "x")
            .expect_err("nonexistent helper must fail");
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    /// A helper that exits non-zero is a failure, not a silent success — and
    /// piping into a helper that never reads must not hang.
    #[test]
    fn clipboard_helper_exit_status_is_checked() {
        assert!(run_clipboard_cmd("false", &[], "payload").is_err());
        assert!(run_clipboard_cmd("true", &[], "payload").is_ok());
    }

    /// The same, with a payload no pipe buffer can hold, so the helper exits
    /// before the write finishes every time rather than only when the runner
    /// loses the race: `false` is still a failure and `true` is still a copy.
    ///
    /// A megabyte is over every platform's pipe capacity (Linux 64 KiB,
    /// macOS 16 KiB), so `write_all` is guaranteed to reach EPIPE here — which
    /// is exactly the state the exit status has to be read over.
    #[test]
    fn a_helper_that_never_reads_is_judged_by_its_exit_status() {
        let payload = "x".repeat(1 << 20);
        assert!(run_clipboard_cmd("true", &[], &payload).is_ok());
        assert!(run_clipboard_cmd("false", &[], &payload).is_err());
    }

    #[test]
    fn osc52_sequence_encodes_and_wraps_for_tmux() {
        assert_eq!(osc52_sequence("foobar", false), "\x1b]52;c;Zm9vYmFy\x07");
        assert_eq!(
            osc52_sequence("foobar", true),
            "\x1bPtmux;\x1b\x1b]52;c;Zm9vYmFy\x07\x1b\\"
        );
    }
}
