```
 ██╗███████╗████████╗ ██████╗ ██████╗ ██████╗ ███████╗
 ██║██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗██╔══██╗██╔════╝
 ██║█████╗     ██║   ██║   ██║██████╔╝██████╔╝╚█████╗
 ██║██╔══╝     ██║   ██║   ██║██╔═══╝ ██╔══██╗ ╚═══██╗
 ██║██║        ██║   ╚██████╔╝██║     ██║  ██║██████╔╝
 ╚═╝╚═╝        ╚═╝    ╚═════╝ ╚═╝     ╚═╝  ╚═╝╚═════╝
```

<p align="center">
  <a href="https://github.com/MenkeTechnologies/iftoprs/actions/workflows/ci.yml"><img src="https://github.com/MenkeTechnologies/iftoprs/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://crates.io/crates/iftoprs"><img src="https://img.shields.io/crates/v/iftoprs.svg" alt="crates.io"></a>
  <a href="https://crates.io/crates/iftoprs"><img src="https://img.shields.io/crates/d/iftoprs.svg" alt="downloads"></a>
  <a href="https://docs.rs/iftoprs"><img src="https://docs.rs/iftoprs/badge.svg" alt="docs.rs"></a>
  <a href="https://github.com/MenkeTechnologies/iftoprs/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/iftoprs.svg" alt="license"></a>
</p>

<p align="center">
  <code>[ SYSTEM://NET_INTERCEPT v2.0 ]</code><br>
  <code>⟦ JACKING INTO YOUR PACKET STREAM ⟧</code><br><br>
  <strong>A neon-drenched terminal UI for real-time bandwidth monitoring</strong><br>
  <em>Built in Rust with <a href="https://github.com/ratatui/ratatui">ratatui</a> + <a href="https://github.com/crossterm-rs/crossterm">crossterm</a> + <a href="https://docs.rs/pcap">pcap</a></em><br><br>
  <code>created by MenkeTechnologies</code>
</p>

<p align="center">
  <img src="screenshots/cli-help.png" alt="CLI Help — iftoprs --help" width="800">
</p>


```bash
cargo install iftoprs
```

---

```
 ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄
 █ >> INITIALIZING PACKET INTERCEPT...                 █
 █ >> STATUS: ALL INTERFACES NOMINAL                   █
 ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
```

### `> FEATURE_DUMP.exe`

```
[CAPTURE_ENGINE]
  ├── Live packet capture ─── libpcap / BPF filters
  │   ├── per-flow bandwidth tracking
  │   ├── sliding window averages: 2s / 10s / 40s
  │   ├── cumulative + peak counters
  │   └── async capture via tokio + mpsc channels
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
  │   ├── lsof-based socket→process mapping
  │   ├── per-process aggregated bandwidth view (Tab key)
  │   └── drill-down: Enter on process → filtered flows, Esc to clear
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
  │   ├── includes rates, totals, process info
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

### `> RENDER_PREVIEW.dat`

#### `// LIVE_CAPTURE`

<p align="center">
  <img src="screenshots/main-view.png" alt="Live Capture View" width="800">
</p>

---

### `> REQUIRED_IMPLANTS.cfg`

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
| `tokio` 1.50 | Async runtime + channels |
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

### `> COMPILE_SEQUENCE.sh`

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

### `> CI_AND_QA.sh`

[GitHub Actions](https://github.com/MenkeTechnologies/iftoprs/actions/workflows/ci.yml) runs on every push and pull request to `main`, and can be started manually (**workflow_dispatch** from the Actions tab).

| Job | Command |
|:---:|:---|
| Format | `cargo --locked fmt --all --check` |
| Clippy | `cargo clippy --all-targets --locked -- -D warnings` |
| Test | `cargo build --locked` and `cargo test --locked` |

Integration tests in **`tests/integration.rs`** execute the built **`iftoprs`** binary via **`CARGO_BIN_EXE_iftoprs`** (not `cargo run`), so CLI output is read directly from the process and stays reliable in CI.

The **Test** job uses **Ubuntu** and **macOS** runners. On Linux, **apt** installs **`libpcap-dev`** for the **Clippy** and **Test** jobs (the **Format** job does not link `pcap` and does not install it). The repo [`rust-toolchain.toml`](rust-toolchain.toml) pins **stable** Rust with `rustfmt` and `clippy` so local and CI toolchains stay aligned. The workflow uses **least-privilege** `contents: read` permissions and **cancels in-progress runs** on the same branch when a newer commit is pushed, so redundant builds do not pile up. Jobs have **timeouts** (format, clippy, and test) so hung runners do not run indefinitely. The test matrix sets **fail-fast: false** so both operating systems finish even when one fails, which makes cross-platform regressions easier to diagnose.

Run the same checks locally before pushing:

```bash
cargo --locked fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

**Testing:** Unit tests in `src/` focus on **deterministic** behavior (flow keys, packet and **`/etc/services`** parsing, bandwidth history, preferences, **TUI** helpers such as **`trunc`** and **rate→bar** scaling in `ui/render.rs`). **Linux** and **macOS** each run **additional** unit tests for OS-specific helpers (`parse_proc_net_port` vs `extract_local_port`). Integration tests in **`tests/integration.rs`** drive the real **`iftoprs`** binary via **`CARGO_BIN_EXE_iftoprs`**.

**Test output:** `cargo test` prints one **`running N tests`** line per test binary (library, **`iftoprs`** binary harness, and integration crate). Summing those three numbers is the total **test executions** in one full run. The library and binary harnesses both run the crate’s unit tests (with a small difference if `main.rs` adds extra tests), so that sum is **not** a count of unique `#[test]` functions.

**`Cargo.lock` is committed** to the repository (this is an application, not a library-only crate) so clean checkouts and CI always have a lockfile. CI uses **`--locked`** so the build fails if `Cargo.lock` is out of date with `Cargo.toml` instead of silently refreshing lockfile entries on the runner. After changing dependencies in `Cargo.toml`, run `cargo build` or `cargo update` locally and commit the updated `Cargo.lock`.

The **actions/cache** keys hash **`Cargo.lock`** and **`rust-toolchain.toml`**, so upgrading the pinned toolchain or changing dependencies invalidates old `target/` artifacts instead of reusing a stale build. Each cache step also sets **`restore-keys`** to a runner-specific prefix so a prior job’s cache can partially warm the next build when the exact key misses. The **Test** job sets **`RUST_BACKTRACE=1`** so panics print useful stack traces in CI logs.

**Port-to-service** labels read the system **`/etc/services`** file; entries and ordering vary by OS. Unit tests use **`tests/fixtures/minimal_etc_services.txt`** for deterministic expectations, and the parser normalizes **`tcp`** / **`udp`** protocol casing so lookups stay consistent. Additional resolver tests cover **port 65535**, **input without a trailing newline**, and **slashes inside the service name token**. A few resolver smoke tests assert against the live file when it is readable and **return early** when it is missing or empty (minimal containers, Windows), so CI stays green without pinning OS-specific service names.

Additional **packet parser** and **`/etc/services` parser** unit tests cover VLAN and cooked (**SLL**) frames, ICMP (no L4 ports), invalid IP versions, **BSD loopback** (**DLT_NULL**) rejects unknown or zero address families, and CIDR edge cases (including **`/0`**, which matches the full IPv4 or IPv6 space for same-family addresses) so regressions show up on **Ubuntu** and **macOS** without live capture hardware.

---

### `> CLI_OPTIONS.exe`

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
```

---

### `> KEYBIND_MATRIX.dat`

```
 ┌──────────────────────────────────────────────────┐
 │           ◈◈◈  COMMAND INTERFACE  ◈◈◈            │
 └──────────────────────────────────────────────────┘
```

#### `// DISPLAY_MODS`

| `KEY` | `ACTION` |
|:---:|:---|
| `Tab` | Switch view ── Flows / Processes |
| `n` | Toggle DNS resolution |
| `N` | Toggle service name resolution |
| `t` | Cycle line display ── two-line / one-line / sent / recv |
| `p` | Toggle port display |
| `Z` | Toggle process display |
| `b` | Toggle bar graphs |
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
| `Esc` | Deselect / clear process filter / close overlay |
| `Enter` | Drill into selected process (Processes tab) |

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
| `b` | Cycle bar style ── gradient / solid / thin / ascii |

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

<p align="center">
  <code>⟦ END OF LINE ⟧</code><br>
  <code>// THE STREET FINDS ITS OWN USES FOR BANDWIDTH //</code>
</p>
