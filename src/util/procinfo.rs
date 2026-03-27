// macOS process attribution via libproc
use std::net::IpAddr;

use crate::data::flow::Protocol;

#[cfg(target_os = "macos")]
mod ffi {
    use std::os::raw::{c_char, c_int, c_void};

    pub const PROC_ALL_PIDS: u32 = 1;
    pub const PROC_PIDFDSOCKETINFO: c_int = 3;

    #[repr(C)]
    #[derive(Clone)]
    pub struct proc_fdinfo {
        pub proc_fd: i32,
        pub proc_fdtype: u32,
    }

    pub const PROX_FDTYPE_SOCKET: u32 = 2;

    // Socket info structures
    #[repr(C)]
    pub struct socket_fdinfo {
        pub pfi: proc_fileinfo,
        pub psi: socket_info,
    }

    #[repr(C)]
    pub struct proc_fileinfo {
        pub fi_openflags: u32,
        pub fi_status: u32,
        pub fi_offset: i64,
        pub fi_type: i32,
        pub fi_guardflags: u32,
    }

    #[repr(C)]
    pub struct socket_info {
        pub soi_stat: soi_stat,
        pub soi_so: u64,
        pub soi_pcb: u64,
        pub soi_type: c_int,
        pub soi_protocol: c_int,
        pub soi_family: c_int,
        pub soi_options: i16,
        pub soi_linger: i16,
        pub soi_state: i16,
        pub soi_qlen: i16,
        pub soi_incqlen: i16,
        pub soi_qlimit: i16,
        pub soi_timeo: i16,
        pub soi_error: u16,
        pub soi_oobmark: u32,
        pub soi_rcv: sockbuf_info,
        pub soi_snd: sockbuf_info,
        pub soi_kind: c_int,
        pub soi_pad: u32,
        pub soi_proto: soi_proto,
    }

    #[repr(C)]
    pub struct soi_stat {
        pub _fields: [u8; 48],
    }

    #[repr(C)]
    pub struct sockbuf_info {
        pub _fields: [u8; 24],
    }

    #[repr(C)]
    pub union soi_proto {
        pub pri_in: in_sockinfo,
        pub pri_tcp: tcp_sockinfo,
        pub pri_un: [u8; 640],
        pub pri_ndrv: [u8; 16],
        pub pri_kern_event: [u8; 48],
        pub pri_kern_ctl: [u8; 288],
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct in_sockinfo {
        pub insi_fport: c_int,   // foreign port
        pub insi_lport: c_int,   // local port
        pub insi_gencnt: u64,
        pub insi_flags: u32,
        pub insi_flow: u32,
        pub insi_vflag: u8,      // INI_IPV4 = 1, INI_IPV6 = 2
        pub insi_ip_ttl: u8,
        pub insi_tpi: u32,
        pub insi_faddr: in_addr_storage,  // foreign addr
        pub insi_laddr: in_addr_storage,  // local addr
        pub insi_v4: in4in6_addr_compat,
        pub insi_v6: in4in6_addr_compat,
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct tcp_sockinfo {
        pub tcpsi_ini: in_sockinfo,
        pub tcpsi_state: c_int,
        pub _pad: [u8; 128],
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct in_addr_storage {
        pub ias_pad: [u8; 16],
    }

    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct in4in6_addr_compat {
        pub _pad: [u8; 16],
    }

    unsafe extern "C" {
        pub fn proc_listpids(
            type_: u32,
            typeinfo: u32,
            buffer: *mut c_void,
            buffersize: c_int,
        ) -> c_int;

        pub fn proc_pidinfo(
            pid: c_int,
            flavor: c_int,
            arg: u64,
            buffer: *mut c_void,
            buffersize: c_int,
        ) -> c_int;

        pub fn proc_name(
            pid: c_int,
            buffer: *mut c_char,
            buffersize: u32,
        ) -> c_int;

        pub fn proc_pidfdinfo(
            pid: c_int,
            fd: c_int,
            flavor: c_int,
            buffer: *mut c_void,
            buffersize: c_int,
        ) -> c_int;
    }

    pub const PROC_PIDLISTFDS: c_int = 1;
}

#[cfg(target_os = "macos")]
pub fn lookup_process(
    _src: IpAddr,
    src_port: u16,
    _dst: IpAddr,
    dst_port: u16,
    protocol: &Protocol,
) -> Option<(u32, String)> {
    use std::mem;
    use std::os::raw::{c_char, c_int, c_void};

    let proto_num: c_int = match protocol {
        Protocol::Tcp => 6,
        Protocol::Udp => 17,
        _ => return None,
    };

    // List all PIDs
    let buf_size = unsafe {
        ffi::proc_listpids(ffi::PROC_ALL_PIDS, 0, std::ptr::null_mut(), 0)
    };
    if buf_size <= 0 {
        return None;
    }

    let num_pids = buf_size as usize / mem::size_of::<c_int>();
    let mut pids = vec![0i32; num_pids];
    let ret = unsafe {
        ffi::proc_listpids(
            ffi::PROC_ALL_PIDS,
            0,
            pids.as_mut_ptr() as *mut c_void,
            buf_size,
        )
    };
    if ret <= 0 {
        return None;
    }
    let actual_count = ret as usize / mem::size_of::<c_int>();

    for &pid in &pids[..actual_count] {
        if pid <= 0 {
            continue;
        }

        // List FDs for this PID
        let fd_buf_size = unsafe {
            ffi::proc_pidinfo(
                pid,
                ffi::PROC_PIDLISTFDS,
                0,
                std::ptr::null_mut(),
                0,
            )
        };
        if fd_buf_size <= 0 {
            continue;
        }

        let num_fds = fd_buf_size as usize / mem::size_of::<ffi::proc_fdinfo>();
        let mut fds = vec![
            ffi::proc_fdinfo {
                proc_fd: 0,
                proc_fdtype: 0,
            };
            num_fds
        ];
        let ret = unsafe {
            ffi::proc_pidinfo(
                pid,
                ffi::PROC_PIDLISTFDS,
                0,
                fds.as_mut_ptr() as *mut c_void,
                fd_buf_size,
            )
        };
        if ret <= 0 {
            continue;
        }

        let actual_fds = ret as usize / mem::size_of::<ffi::proc_fdinfo>();
        for fd_info in &fds[..actual_fds] {
            if fd_info.proc_fdtype != ffi::PROX_FDTYPE_SOCKET {
                continue;
            }

            let mut sock_info: ffi::socket_fdinfo = unsafe { mem::zeroed() };
            let ret = unsafe {
                ffi::proc_pidfdinfo(
                    pid,
                    fd_info.proc_fd,
                    ffi::PROC_PIDFDSOCKETINFO,
                    &mut sock_info as *mut _ as *mut c_void,
                    mem::size_of::<ffi::socket_fdinfo>() as c_int,
                )
            };
            if ret <= 0 {
                continue;
            }

            // Check if this socket matches our flow
            let si = &sock_info.psi;
            if si.soi_protocol != proto_num {
                continue;
            }
            // AF_INET = 2, AF_INET6 = 30
            if si.soi_family != 2 && si.soi_family != 30 {
                continue;
            }

            let in_si = unsafe { &si.soi_proto.pri_in };
            let local_port = in_si.insi_lport as u16;
            let foreign_port = in_si.insi_fport as u16;

            // Match: local is src, foreign is dst (or vice versa)
            let matches = (local_port == src_port && foreign_port == dst_port)
                || (local_port == dst_port && foreign_port == src_port);

            if matches {
                // Get process name
                let mut name_buf = [0u8; 256];
                let name_len = unsafe {
                    ffi::proc_name(
                        pid,
                        name_buf.as_mut_ptr() as *mut c_char,
                        name_buf.len() as u32,
                    )
                };
                let name = if name_len > 0 {
                    String::from_utf8_lossy(&name_buf[..name_len as usize]).to_string()
                } else {
                    format!("pid:{}", pid)
                };
                return Some((pid as u32, name));
            }
        }
    }

    None
}

#[cfg(not(target_os = "macos"))]
pub fn lookup_process(
    _src: IpAddr,
    _src_port: u16,
    _dst: IpAddr,
    _dst_port: u16,
    _protocol: &Protocol,
) -> Option<(u32, String)> {
    None
}
