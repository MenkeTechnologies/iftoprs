```
 ██╗███████╗████████╗ ██████╗ ██████╗ ██████╗ ███████╗
 ██║██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗██╔══██╗██╔════╝
 ██║█████╗     ██║   ██║   ██║██████╔╝██████╔╝╚█████╗
 ██║██╔══╝     ██║   ██║   ██║██╔═══╝ ██╔══██╗ ╚═══██╗
 ██║██║        ██║   ╚██████╔╝██║     ██║  ██║██████╔╝
 ╚═╝╚═╝        ╚═╝    ╚═════╝ ╚═╝     ╚═╝  ╚═╝╚═════╝
```

[![CI](https://github.com/MenkeTechnologies/iftoprs/actions/workflows/ci.yml/badge.svg)](https://github.com/MenkeTechnologies/iftoprs/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/iftoprs.svg)](https://crates.io/crates/iftoprs)
[![Downloads](https://img.shields.io/crates/d/iftoprs.svg)](https://crates.io/crates/iftoprs)
[![Docs.rs](https://docs.rs/iftoprs/badge.svg)](https://docs.rs/iftoprs)
 [![Docs](https://img.shields.io/badge/docs-online-blue.svg)](https://menketechnologies.github.io/iftoprs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

### `[SYSTEM://NET_INTERCEPT // JACKING INTO YOUR PACKET STREAM]`

> *"The street finds its own uses for bandwidth."*

A neon-drenched terminal UI for real-time bandwidth monitoring. Built in Rust with [ratatui](https://github.com/ratatui/ratatui) + [crossterm](https://github.com/crossterm-rs/crossterm) + [pcap](https://docs.rs/pcap). 31 cyberpunk themes, native process attribution (no external tools), JSON streaming, BPF filters, mouse + sparklines, auto-restart capture, hover tooltips.

```bash
brew tap MenkeTechnologies/menketech    # one-time
brew install iftoprs                    # via Homebrew tap (recommended)

cargo install iftoprs                   # via crates.io
```

<p align="center">
  <img src="screenshots/cli-help.png" alt="CLI Help — iftoprs --help" width="800">
</p>

### [`Read the Docs`](https://menketechnologies.github.io/iftoprs/) &middot; [`Engineering Report`](https://menketechnologies.github.io/iftoprs/report.html) · [`strykelang`](https://github.com/MenkeTechnologies/strykelang) · [`zshrs`](https://github.com/MenkeTechnologies/zshrs) · [`lsofrs`](https://github.com/MenkeTechnologies/lsofrs)

---

## Table of Contents

- [\[0x00\] Feature Dump](#0x00-feature-dump)
- [\[0x01\] Render Preview](#0x01-render-preview)
- [\[0x02\] Required Implants](#0x02-required-implants)
- [\[0x03\] Compile Sequence](#0x03-compile-sequence)
- [\[0x04\] CI & QA](#0x04-ci--qa)
- [\[0x05\] CLI Options](#0x05-cli-options)
- [\[0x06\] Keybind Matrix](#0x06-keybind-matrix)
- [\[0xFF\] License](#0xff-license)

---

```
 ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄
 █ >> INITIALIZING PACKET INTERCEPT...                 █
 █ >> STATUS: ALL INTERFACES NOMINAL                   █
 ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
```

## [0x00] FEATURE DUMP

```
[CAPTURE_ENGINE]
  ├── Live packet capture ─── libpcap / BPF filters
  │   ├── per-flow bandwidth tracking
  │   ├── sliding window averages: 2s / 10s / 40s
  │   ├── cumulative + peak counters
  │   ├── async capture via tokio + mpsc channels
  │   └── auto-restart on transient errors (exponential backoff)
  │
[TELEMETRY_CORE]
  ├── Real-time flow analysis
  │   ├── source ↔ destination pair tracking
  │   ├── protocol detection: TCP / UDP / ICMP / Other
  │   ├── DNS reverse resolution (async, cached)
  │   ├── port-to-service name mapping
  │   └── log10 bandwidth scale: 10b → 1Gb
  │
[PROCESS_INTEL]
  ├── Flow-to-process attribution
  │   ├── PID + process name per connection
  │   ├── background polling via Arc<Mutex<>>
  │   ├── native socket→process mapping (libproc on macOS, /proc on Linux)
  │   ├── per-process aggregated bandwidth view (Tab key)
  │   └── drill-down: Enter on process → filtered flows, Esc to clear
  │
[PROVENANCE_INTEL]
  ├── Publishers view ── third aggregation axis (Tab key)
  │   ├── resolves each flow past pid→name to the binary's code identity
  │   ├── macOS: code-signing Team ID + authority (Security framework)
  │   ├── Linux: owning package (dpkg/rpm) + executable SHA-256
  │   ├── rolls TX/RX up by publisher ("unsigned-binary 40 Mb/s up")
  │   ├── cache keyed by (dev, inode, mtime) — one fingerprint per binary
  │   └── drill-down: Enter on publisher → filtered flows, Esc to clear
  │
[TOOLTIP_SYSTEM]
  ├── Rich contextual tooltips on hover + right-click
  │   ├── right-click flow rows ── TX/RX rates, totals, process, sparkline
  │   ├── hover header bar segments ── 1s delay, 3s auto-hide
  │   ├── right-click header ── instant tooltip, persistent until dismissed
  │   ├── segment tooltips: app info, interface, flows, clock, sort,
  │   │   refresh rate, theme, filter, paused state, help
  │   └── 9-15 lines per segment: config fields, sources, key hints
  │
[SPARKLINE]
  ├── Per-flow bandwidth sparkline (▁▂▃▅▇█)
  │   ├── shown on row below selected flow (40s history)
  │   └── shown in right-click tooltip
  │
[JSON_STREAM]
  ├── --json flag ── headless NDJSON output (no TUI)
  │   ├── streams flow snapshots to stdout
  │   ├── includes rates, totals, process info, publisher/team_id/package
  │   └── pipe to jq, log to file, feed dashboards
  │
[INTERFACE_DECK]
  ├── Sort ─── 2s avg / 10s avg / 40s avg / src name / dst name
  ├── Display ─── bits or bytes / bars on/off / ports on/off
  ├── Line modes ─── two-line / one-line / sent-only / recv-only
  ├── Freeze ─── lock current sort order
  └── Color-coded rate columns ─── yellow(2s) / green(10s) / cyan(40s)
  │
[NET_FILTER]
  ├── BPF filter expressions ─── "tcp port 80", "host 10.0.0.1"
  ├── CIDR network filter ─── auto-detect or manual (-F)
  ├── Promiscuous mode ─── capture all traffic on segment
  └── Interface selection ─── list + choose
  │
[PLATFORM_COMPAT]
  ├── macOS ── SUPPORTED
  ├── Linux ── SUPPORTED
  └── requires libpcap (root/sudo for raw capture)
  │
[THEME_ENGINE]
  ├── 31 builtin cyberpunk color themes (including iftopcolor)
  │   ├── live theme chooser (c key)
  │   ├── swatch preview per theme
  │   └── persistent selection via ~/.iftoprs.conf
  │
[FLOW_SELECTION]
  ├── j/k ── select next/prev flow
  ├── Ctrl+d/u ── half-page scroll
  ├── G/Home ── jump to last/first
  ├── y ── copy selected flow to clipboard
  ├── F ── pin/unpin flow (★ floats to top)
  └── Esc ── deselect
  │
[FILTER_ENGINE]
  ├── / ── live filter by hostname/IP
  ├── 0 ── clear filter
  ├── Ctrl+w ── delete word
  └── Ctrl+k ── kill to end of line
  │
[EXPORT]
  ├── e ── export all flows to ~/.iftoprs.export.txt
  └── includes per-flow rates + TX/RX totals
  │
[ALERT_SYSTEM]
  ├── Bandwidth threshold alerts ── configurable in ~/.iftoprs.conf
  │   ├── red border flash on threshold crossing
  │   ├── terminal bell (\x07) notification
  │   └── status bar message: ⚠ ALERT: hostname rate/s
  │
[CONFIG_ENGINE]
  ├── Auto-save ── every toggle writes to ~/.iftoprs.conf
  ├── Default config ── created on first run if missing
  ├── Reference config ── iftoprs.default.conf with full docs
  └── TOML format ── human-readable, hand-editable
  │
[SHELL_COMPLETION]
  ├── Zsh completions ── completions/_iftoprs
  └── --completions flag ── zsh / bash / fish / elvish / powershell
```

---

## [0x01] RENDER PREVIEW

#### `// LIVE_CAPTURE`

<p align="center">
  <img src="screenshots/main-view.png" alt="Live Capture View" width="800">
</p>

---

## [0x02] REQUIRED IMPLANTS

```
RUST_VERSION  >= 1.85  [2024 edition]
TARGET_OS     == macOS || Linux
LIBPCAP       == installed (system dependency)
```

| `IMPLANT` | `PURPOSE` |
|:---:|:---|
| `ratatui` 0.30 | TUI rendering framework |
| `crossterm` 0.29 | Terminal events + manipulation |
| `pcap` 2.4 | Packet capture via libpcap |
| `tokio` 1.51 | Async runtime + channels |
| `clap` 4.6 | CLI argument parsing |
| `dns-lookup` 3.0 | Reverse DNS resolution |
| `regex` 1.12 | Pattern matching for filters |
| `chrono` 0.4 | Time operations |
| `anyhow` 1.0 | Error handling |
| `clap_complete` 4 | Shell completion generation |
| `serde` 1.0 | Config serialization |
| `serde_json` 1.0 | JSON streaming output |
| `toml` 1.1 | Config file format |
| `dirs` 6.0 | Home directory detection |

---

## [0x03] COMPILE SEQUENCE

```bash
# ── JACK IN ──────────────────────────────────
cargo build --release
# LTO enabled ── symbols stripped ── lean binary
```

```bash
# ── BOOT THE SNIFFER ─────────────────────────
sudo cargo run --release
# or go direct:
sudo ./target/release/iftoprs
```

```bash
# ── INSTALL MAN PAGES ────────────────────────
sudo cp man/man1/iftoprs.1    /usr/local/share/man/man1/
sudo cp man/man1/iftoprsall.1 /usr/local/share/man/man1/
man iftoprs        # short reference
man iftoprsall     # full reference (keybindings, themes, architecture)
```

## [0x04] CI & QA

[GitHub Actions](https://github.com/MenkeTechnologies/iftoprs/actions/workflows/ci.yml) runs on every push and pull request to `main`, and can be started manually (**workflow_dispatch** from the Actions tab).

| Job | Command |
|:---:|:---|
| Format | `cargo --locked fmt --all --check` |
| Clippy | `cargo clippy --all-targets --locked -- -D warnings` |
| Doc | `cargo doc --locked --no-deps` with `RUSTDOCFLAGS=-D warnings` |
| Test | `cargo build --verbose --locked` and `cargo test --verbose --locked` |
| Docs / Polish / Semantic / Newline + README / Structure gates | shell gate scripts under `tests/*.sh` (docs HTML hygiene, man pages, final newlines, README structure) |

Integration tests in **`tests/integration.rs`** execute the built **`iftoprs`** binary via **`CARGO_BIN_EXE_iftoprs`** (not `cargo run`), so CLI output is read directly from the process and stays reliable in CI.

The **Test** job uses **Ubuntu** and **macOS** runners. On Linux, **apt** installs **`libpcap-dev`** for the **Clippy**, **Doc**, and **Test** jobs (the **Format** job does not link `pcap` and does not install it). The repo [`rust-toolchain.toml`](rust-toolchain.toml) pins **stable** Rust with `rustfmt` and `clippy` so local and CI toolchains stay aligned. The workflow uses **least-privilege** `contents: read` permissions and **cancels in-progress runs** on the same branch when a newer commit is pushed, so redundant builds do not pile up. Jobs have **timeouts** (format, clippy, and test) so hung runners do not run indefinitely. The test matrix sets **fail-fast: false** so both operating systems finish even when one fails, which makes cross-platform regressions easier to diagnose.

Run the same checks locally before pushing:

```bash
cargo --locked fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

---

## [0x05] CLI OPTIONS

```
 ┌──────────────────────────────────────────────────┐
 │           ◈◈◈  COMMAND LINE DECK  ◈◈◈            │
 └──────────────────────────────────────────────────┘
```

#### `// CAPTURE`

| `FLAG` | `DESCRIPTION` |
|:---|:---|
| `-i, --interface NAME` | Network interface to monitor |
| `-f, --filter EXPR` | BPF filter expression (e.g., "tcp port 80") |
| `-F, --net-filter CIDR` | IPv4 network filter (e.g., "192.168.1.0/24") |
| `-p, --promiscuous` | Enable promiscuous mode |

#### `// DISPLAY`

| `FLAG` | `DESCRIPTION` |
|:---|:---|
| `-n, --no-dns` | Disable DNS hostname resolution |
| `-N, --no-port-names` | Disable port-to-service resolution |
| `-b, --no-bars` | Disable bar graph display |
| `-B, --bytes` | Display bandwidth in bytes (instead of bits) |
| `-P, --hide-ports` | Hide ports alongside hosts |
| `-Z, --no-processes` | Hide process column (shown by default) |

#### `// OUTPUT`

| `FLAG` | `DESCRIPTION` |
|:---|:---|
| `--json` | Stream NDJSON to stdout (no TUI) |

#### `// SYSTEM`

| `FLAG` | `DESCRIPTION` |
|:---|:---|
| `-c, --config FILE` | Path to config file (default: ~/.iftoprs.conf) |
| `-l, --list-interfaces` | List available interfaces and exit |
| `--list-colors` | Preview all 31 color themes with swatches |
| `--completions SHELL` | Generate shell completions (zsh, bash, fish, elvish, powershell) |
| `-h, --help` | Display help transmission |
| `-V, --version` | Display version information |

#### `// EXAMPLES`

```bash
sudo iftoprs -i en0                    # monitor specific interface
sudo iftoprs -f "tcp port 443"         # filter HTTPS traffic only
sudo iftoprs -F 10.0.0.0/8 -B         # filter private net, show bytes
sudo iftoprs -n -N -b                  # raw IPs, no bars, minimal
sudo iftoprs -Z                        # show process names per flow
sudo iftoprs -p                        # promiscuous mode
iftoprs --completions zsh              # generate zsh completions
sudo iftoprs --json                    # stream NDJSON to stdout
sudo iftoprs --json | jq '.flows[0]'  # pipe to jq for processing
sudo iftoprs --json | jq '.flows[] | {publisher, team_id, package}'  # code provenance
```

Each NDJSON flow carries the usual rates/totals plus process attribution
(`process_name`, `pid`) and code provenance. Provenance fields are emitted only
when resolved: `publisher` (rollup label — Team ID, package, or a verdict such
as `unsigned-binary`), `team_id` (macOS code-signing Team Identifier), and
`package` (Linux dpkg/rpm owning package).

---

## [0x06] KEYBIND MATRIX

```
 ┌──────────────────────────────────────────────────┐
 │           ◈◈◈  COMMAND INTERFACE  ◈◈◈            │
 └──────────────────────────────────────────────────┘
```

#### `// DISPLAY_MODS`

| `KEY` | `ACTION` |
|:---:|:---|
| `Tab` | Switch view ── Flows / Processes / Publishers |
| `n` | Toggle DNS resolution |
| `N` | Toggle service name resolution |
| `t` | Cycle line display ── two-line / one-line / sent / recv |
| `p` | Toggle port display |
| `Z` | Toggle process display |
| `b` | Cycle bar style (gradient / solid / thin / ascii) |
| `B` | Toggle bytes/bits |
| `T` | Toggle hover tooltips (right-click still works) |
| `U` | Toggle cumulative totals |
| `P` | Pause / resume display (shows overlay) |
| `x` | Toggle border chrome |
| `g` | Toggle column header |
| `f` | Cycle refresh rate ── 1s / 2s / 5s / 10s |

#### `// SORT_PROTOCOL`

| `KEY` | `ACTION` |
|:---:|:---|
| `1` | Sort by 2s average |
| `2` | Sort by 10s average |
| `3` | Sort by 40s average |
| `<` | Sort by source name |
| `>` | Sort by destination name |
| `o` | Freeze current sort order |

| `r` | Reverse sort order |

#### `// NAVIGATION`

| `KEY` | `ACTION` |
|:---:|:---|
| `j` `↓` | Select next flow |
| `k` `↑` | Select prev flow |
| `Ctrl+D` | Half-page down |
| `Ctrl+U` | Half-page up |
| `G` `End` | Jump to last |
| `Home` | Jump to first |
| `Esc` | Deselect / clear process or publisher filter / close overlay |
| `Enter` | Drill into selected process / publisher (Processes / Publishers tab) |

#### `// FILTER_OPS`

| `KEY` | `ACTION` |
|:---:|:---|
| `/` | Enter filter mode |
| `0` | Clear filter |
| `Enter` | Confirm filter |
| `Esc` | Cancel filter |

#### `// THEME_OPS`

| `KEY` | `ACTION` |
|:---:|:---|
| `c` | Open theme chooser |
| `j/k` | Navigate themes |
| `Enter` | Select theme |
| `Esc` | Cancel |

#### `// INTERFACE_OPS`

| `KEY` | `ACTION` |
|:---:|:---|
| `i` | Open interface chooser (also cycles in popup) |
| `j/k` | Navigate interfaces |
| `Enter` | Select interface (saved to config, restart to apply) |
| `Esc` | Cancel |

#### `// ACTIONS`

| `KEY` | `ACTION` |
|:---:|:---|
| `y` | Copy selected flow to clipboard |
| `F` | Pin/unpin selected flow ★ |
| `e` | Export flows to ~/.iftoprs.export.txt |
#### `// MOUSE`

| `INPUT` | `ACTION` |
|:---:|:---|
| Left click | Select flow row |
| Right click (flow) | Show TX/RX tooltip with bandwidth, process, sparkline |
| Right click (header) | Instant segment tooltip (persistent until dismissed) |
| Middle click | Pin/unpin flow |
| Mouse move | Dismiss flow tooltip |
| Scroll up/down | Navigate flows (cycle themes in chooser) |
| Hover header bar | Segment tooltip after 1s delay (auto-hides after 3s) |

#### `// GENERAL_OPS`

| `KEY` | `ACTION` |
|:---:|:---|
| `h` `?` | Toggle help HUD |
| `q` | Disconnect (saves prefs) |
| `Ctrl+C` | Force disconnect |

---

## [0xFF] LICENSE

MIT License — MenkeTechnologies. See [LICENSE](LICENSE).

<p align="center">
  <code>⟦ END OF LINE ⟧</code><br>
  <code>// THE STREET FINDS ITS OWN USES FOR BANDWIDTH //</code>
</p>
