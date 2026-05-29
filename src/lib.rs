//! `iftoprs` library surface — real-time bandwidth monitor (iftop clone) built
//! on `ratatui` + `pcap`. The crate is consumed by the binary in `main.rs`
//! and by the integration tests in `tests/`.
/// `capture` submodule.
pub mod capture;
/// `config` submodule.
pub mod config;
/// `data` submodule.
pub mod data;
/// `ui` submodule.
pub mod ui;
/// `util` submodule.
pub mod util;
