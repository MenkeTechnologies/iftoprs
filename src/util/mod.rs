pub mod format;
pub mod resolver;

#[cfg(target_os = "macos")]
pub mod procinfo;
#[cfg(target_os = "linux")]
pub mod procinfo_linux;

// Re-export a unified interface
#[cfg(target_os = "macos")]
pub use procinfo::lookup_process;

#[cfg(target_os = "linux")]
pub use procinfo_linux::lookup_process;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn lookup_process(
    _src: std::net::IpAddr,
    _src_port: u16,
    _dst: std::net::IpAddr,
    _dst_port: u16,
    _protocol: &crate::data::flow::Protocol,
) -> Option<(u32, String)> {
    None
}
