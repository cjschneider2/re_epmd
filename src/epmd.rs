#![allow(dead_code, unused_imports, unused_variables)]

/// Citations:
/// [1]: https://msdn.microsoft.com/en-us/library/windows/desktop/ms741563.aspx
/// [2]: https://lists.fedoraproject.org/pipermail/devel/2010-July/139135.html

use std::collections::HashSet;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::net::TcpListener;
use std::net::ToSocketAddrs;
use std::str::FromStr;
use std::mem::uninitialized;


use net2::TcpBuilder;
use net2::TcpListenerExt;
use net2::unix::UnixTcpBuilderExt;

use libc;

use super::run_daemon;
use connection::Connection;
use constants::{MAX_LISTEN_SOCKETS, CLOSE_TIMEOUT, MAX_FILE_DESCRIPTORS};
use erl_node::ErlNode;

pub struct Epmd {
    // -- program flags --
    pub debug: bool,
    pub silent: bool,
    pub is_daemon: bool,
    pub is_systemd: bool,
    pub brutal_kill: bool, // Check if needed
    pub use_ipv6: bool,
    // -- extra options --
    pub packet_timeout: usize,
    pub delay_accept: usize,
    pub delay_write: usize,
    // -- connection properties --
    pub address: String,
    pub port: u16,
    // -- program constants --
    pub max_conn: usize,
    // TODO: active_conn & listen_fd maybe could be combined into a Vec<fd>
    pub active_conn: usize,
    pub listen_fd: [i32; MAX_LISTEN_SOCKETS],
    // -- program data --
    pub nodes: HashSet<ErlNode>,
    pub conn: Vec<Connection>,
    // -- currently unused --
    pub select_fd_top: usize, // what is this?
    pub orig_read_mask: libc::fd_set,
}


#[cfg(target_pointer_width = "32")]
const POINTER_BITS:usize = 32;
#[cfg(target_pointer_width = "64")]
const POINTER_BITS:usize = 64;

impl Epmd {
    pub fn new () -> Epmd {
        Epmd {
            // -- program flags --
            debug: false,
            silent: false,
            is_daemon: false,
            is_systemd: false,
            brutal_kill: false,
            use_ipv6: false,
            // -- extra options --
            packet_timeout: CLOSE_TIMEOUT,
            delay_accept: 0,
            delay_write: 0,
            // -- program constants --
            max_conn: MAX_FILE_DESCRIPTORS,
            // -- connection properties --
            address: get_address(),
            port: get_port_number(),
            active_conn: 0,
            listen_fd: [-1; MAX_LISTEN_SOCKETS],
            // -- program data --
            nodes: HashSet::<ErlNode>::new(),
            conn: Vec::<Connection>::new(),
            // -- currently unused --
            select_fd_top: 0,
            orig_read_mask: unsafe { uninitialized() },
        }
    }

    pub fn kill (&mut self) {
        println!("TODO: epmd.kill()");
        unimplemented!();
    }

    pub fn call (&mut self) {
        println!("TODO: epmd.call()");
        unimplemented!();
    }

    pub fn run (&mut self) {

        /* TODO: systemd related initialization...
        epmd does some querying of the system though systemd if it's available.
        namely using `sd_listen_fds(0)` To get the max # of sockets of the
        system. [2] has a decent way to go about using systemd to find the
        start of the socket counters in this way. This is set elsewhere so I'll
        just ignore this for now. ( the max will just be `MAX_FILE_DESCRIPTORS` )

        IDEA?: maybe put all of the systemd functions into it's own module?
        */

        let addrs = parse_socket_addrs(&self.address, self.port, self.use_ipv6);
        let num_sockets = addrs.len();

        if num_sockets >= MAX_LISTEN_SOCKETS {
            panic!("Cannot listen on more than {} IP Addresses",
                   MAX_LISTEN_SOCKETS);
        }

        if cfg!(all(unix)) {
            ignore_sig_pipe();
        }

        // Initialize the number of active file descriptors;
        // `stdin`, `stdout`, & `stderr` are still open.
        self.active_conn = 3 + num_sockets;
        self.max_conn -= num_sockets;

        // Initialize variables for select()
        self.init_select_vars();

        // Setup file descriptors
        let mut listen_sock: [i32; MAX_LISTEN_SOCKETS] = [0; MAX_LISTEN_SOCKETS];
        if self.is_systemd {
            // for idx in (0..num_sockets) {
            //     `select_fd_set(self, listensock[i])`
            //}
            unimplemented!()
        } else {
            // TODO: maybe this could be reduce by using `addrs` as an iterator?
            for idx in 0..num_sockets {
                let sock_family = match addrs[idx] {
                    SocketAddr::V4(_) => libc::AF_INET,
                    SocketAddr::V6(_) => libc::AF_INET6
                };
                listen_sock[idx] = unsafe{
                    libc::socket(sock_family, libc::SOCK_STREAM, 0)
                };
                if listen_sock[idx] < 0 {
                    // TODO: Read `errno` to identify the error
                    // IDEA: this read `errno` function should also be a part of
                    // the libc_utilities module...
                }
            }
            // TODO: `g->listenfd[bound++] = listensock[i];`

            // TODO: `setsockopt` for `IPV6_V6ONLY` if `HAVE_DECL_IPV6_ONLY` is
            // selected as a compile time option -> translate into a feature
            // flag for cargo.

            // Set `SO_REUSEADDR` on all non-windows platforms;
            // On windows, if this is set the addresses will be reused even if
            // they are already in use. (behavior difference)
            if !cfg!(target_os = "windows") {
                // TODO: From C:
                //opt = 1;
                //if ( setsockopt(listensock[i], SOL_SOCKET,
                //                SO_REUSEADDR, &opt, sizeof(opt) < 0)){
                //    /* check error */
                //}
                unimplemented!()
            }

            // TODO: this is line 406 or so in `epmd_srv.c`

            unimplemented!()
        }

        unimplemented!();
    }

    pub fn stop(&mut self, val: String) {
        val.len();
        unimplemented!();
    }

    pub fn run_daemon(self) {
        run_daemon(self);
    }

    fn init_select_vars(&mut self) {
        // TODO:
        // Factor all of the select / fd_set stuff into it's own module
        // to put all of the unsafe code in one place.
        //
        // maybe call it `libc_utilities.rs` and include `ignore_sig_pipe()`
        // in with that file...
        unsafe {
            libc::FD_ZERO(&mut self.orig_read_mask);
        }
        self.select_fd_top = 0;
    }


}

/// Ignore the SIGPIPE signal that is raised when we call write
/// twice on a socket closed by the other end.
fn ignore_sig_pipe () {
    use libc::{signal, SIGPIPE, SIG_IGN};
    unsafe { signal(SIGPIPE, SIG_IGN); }
}

/// Creates a list of listening sockets from which we can call select on and
/// check for incoming connections.
fn create_listen_sockets_from_addrs (
    in_socks: Vec<SocketAddr>
) -> Vec<TcpListener> {
    let sockets = Vec::<TcpListener>::new();
    sockets
}

/// `parse_socket_addrs` assumes that the addresses are given in the forms of:
///    "192.168.1.1"
///    "192.168.1.1, 10.0.0.1"
///    "192.168.1.1 10.0.0.1"
/// or their ipv6 counterparts and sets the listen port to the parameter value.
fn parse_socket_addrs (
    addr_str: &str, port: u16, use_ipv6: bool
) -> Vec<SocketAddr> {
    let mut socket_addrs: Vec<SocketAddr> = Vec::new();
    // If we have an address list then we'll use it;
    if !addr_str.is_empty() {
        // Always join the loopback address
        socket_addrs.push(get_loopback_address(port, use_ipv6));
        // Parse the rest of the addresses given to us in the configuration
        let mut addrs: Vec<_> =
            addr_str
            .split(|c| c == ',' || c == ' ')
            .filter_map(|addr| {
                if let Ok(_a) = addr.parse::<Ipv4Addr>() {
                    let _v4 = SocketAddrV4::new(_a, port);
                    Some(SocketAddr::V4(_v4))
                } else if let Ok(_a) = addr.parse::<Ipv6Addr>() {
                    let _v6 = SocketAddrV6::new(_a, port, 0, 0);
                    Some(SocketAddr::V6(_v6))
                } else {
                    None
                }
            })
            .collect();
        socket_addrs.append(&mut addrs);
    } else { // just listen on any address...
        socket_addrs.push(get_any_address(port, use_ipv6));
    }
    socket_addrs
}

fn get_loopback_address(port: u16, use_ipv6: bool) -> SocketAddr {
    if use_ipv6 {
        let _v6 = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
        let _v6 = SocketAddrV6::new(_v6, port, 0, 0);
        SocketAddr::V6(_v6)
    } else {
        let _v4 = Ipv4Addr::new(127, 0, 0, 1);
        let _v4 = SocketAddrV4::new(_v4, port);
        SocketAddr::V4(_v4)
    }
}

fn get_any_address(port: u16, use_ipv6: bool) -> SocketAddr {
    if use_ipv6 {
        let _v6 = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0);
        let _v6 = SocketAddrV6::new(_v6, port, 0, 0);
        SocketAddr::V6(_v6)
    } else {
        let _v4 = Ipv4Addr::new(0, 0, 0, 0);
        let _v4 = SocketAddrV4::new(_v4, port);
        SocketAddr::V4(_v4)
    }
}

fn get_address() -> String {
    use std::env::var;
    var("ERL_EPMD_ADDRESS").unwrap_or("".into())
}

fn get_port_number() -> u16 {
    use std::env::var;
    use constants::EPMD_PORT_NUMBER;
    match var("ERL_EPMD_PORT") {
        Ok(val) => {
            match u16::from_str_radix(&val, 10) {
                Ok(val) => val,
                Err(_) => EPMD_PORT_NUMBER
            }
        },
        Err(_) => EPMD_PORT_NUMBER
    }
}

fn check_relaxed() -> bool {
    use std::env::var;
    match var("ERL_EPMD_RELAXED_COMMAND_CHECK") {
        Ok(_)  => true,
        Err(_) => false
    }
}

#[cfg(test)]
mod tests {

    use std::net::{Ipv6Addr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

    use super::parse_socket_addrs;
    use super::get_any_address;
    use super::get_loopback_address;

    #[test]
    fn test_parse_socket_addrs_blank () {
        let test_str = "";
        let use_ipv6 = false;
        let res = parse_socket_addrs(test_str, 0x1234, use_ipv6);
        println!("{:?}", res);
        assert_eq!(res[0], get_any_address(0x1234, use_ipv6));
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn test_parse_socket_addrs_blank_ipv6 () {
        let test_str = "";
        let use_ipv6 = true;
        let res = parse_socket_addrs(test_str, 0x1234, use_ipv6);
        println!("{:?}", res);
        assert_eq!(res[0], get_any_address(0x1234, use_ipv6));
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn test_parse_socket_addrs_just_commas () {
        let test_str = ",,,";
        let use_ipv6 = false;
        let res = parse_socket_addrs(test_str, 0x1234, use_ipv6);
        println!("{:?}", res);
        assert_eq!(res[0], get_loopback_address(0x1234, use_ipv6));
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn test_parse_socket_addrs_just_commas_ipv6 () {
        let test_str = ",,,";
        let use_ipv6 = true;
        let res = parse_socket_addrs(test_str, 0x1234, use_ipv6);
        println!("{:?}", res);
        assert_eq!(res[0], get_loopback_address(0x1234, use_ipv6));
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn test_parse_socket_addrs_space_seperators () {
        let test_str = "123.123.123.123 234.234.234.234";
        let addr1 = SocketAddr::V4(
            SocketAddrV4::new(Ipv4Addr::new(123,123,123,123), 0x1234));
        let addr2 = SocketAddr::V4(
            SocketAddrV4::new(Ipv4Addr::new(234,234,234,234), 0x1234));
        let use_ipv6 = false;
        let res = parse_socket_addrs(test_str, 0x1234, use_ipv6);
        println!("{:?}", res);
        assert_eq!(res[0], get_loopback_address(0x1234, use_ipv6));
        assert_eq!(res[1], addr1);
        assert_eq!(res[2], addr2);
        assert_eq!(res.len(), 3);
    }

    #[test]
    fn test_parse_socket_addrs_comma_seperators () {
        let test_str = "123.123.123.123, 234.234.234.234";
        let addr1 = SocketAddr::V4(
            SocketAddrV4::new(Ipv4Addr::new(123,123,123,123), 0x1234));
        let addr2 = SocketAddr::V4(
            SocketAddrV4::new(Ipv4Addr::new(234,234,234,234), 0x1234));
        let use_ipv6 = false;
        let res = parse_socket_addrs(test_str, 0x1234, use_ipv6);
        println!("{:?}", res);
        assert_eq!(res[0], get_loopback_address(0x1234, use_ipv6));
        assert_eq!(res[1], addr1);
        assert_eq!(res[2], addr2);
        assert_eq!(res.len(), 3);
    }

    #[test]
    fn test_parse_socket_addrs_comma_seperators_ipv6 () {
        let test_str = "2001:db8::2:1, 2001:db8:85a3::8a2e:370:7334";
        let addr1 = SocketAddr::V6(
            SocketAddrV6::new(
                Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 2, 1), 0x1234, 0, 0));
        let addr2 = SocketAddr::V6(
            SocketAddrV6::new(
                Ipv6Addr::new(0x2001, 0xdb8, 0x85a3, 0, 0, 0x8a2e, 0x370, 0x7334),
                0x1234, 0, 0));
        let use_ipv6 = true;
        let res = parse_socket_addrs(test_str, 0x1234, use_ipv6);
        println!("{:?}", res);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0], get_loopback_address(0x1234, use_ipv6));
        assert_eq!(res[1], addr1);
        assert_eq!(res[2], addr2);
    }
}
