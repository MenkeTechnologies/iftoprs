```
 ██╗███████╗████████╗ ██████╗ ██████╗ ██████╗ ███████╗
 ██║██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗██╔══██╗██╔════╝
 ██║█████╗     ██║   ██║   ██║██████╔╝██████╔╝╚█████╗
 ██║██╔══╝     ██║   ██║   ██║██╔═══╝ ██╔══██╗ ╚═══██╗
 ██║██║        ██║   ╚██████╔╝██║     ██║  ██║██████╔╝
 ╚═╝╚═╝        ╚═╝    ╚═════╝ ╚═╝     ╚═╝  ╚═╝╚═════╝
```

<p align="center">
  <a href="https://crates.io/crates/iftoprs"><img src="https://img.shields.io/crates/v/iftoprs.svg" alt="crates.io"></a>
  <a href="https://crates.io/crates/iftoprs"><img src="https://img.shields.io/crates/d/iftoprs.svg" alt="downloads"></a>
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
  │   └── lsof-based socket→process mapping
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

#### `// SYSTEM`

| `FLAG` | `DESCRIPTION` |
|:---|:---|
| `-l, --list-interfaces` | List available interfaces and exit |
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
| `n` | Toggle DNS resolution |
| `N` | Toggle service name resolution |
| `t` | Cycle line display ── two-line / one-line / sent / recv |
| `p` | Toggle port display |
| `Z` | Toggle process display |
| `b` | Toggle bar graphs |
| `B` | Toggle bytes/bits |
| `T` | Toggle cumulative totals |
| `P` | Pause / resume display |

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
| `Esc` | Deselect / close overlay |

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
| Right click | Show TX/RX tooltip |
| Middle click | Pin/unpin flow |
| Scroll up/down | Navigate flows |

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
