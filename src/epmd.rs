#![allow(dead_code)]

/// Citations:
/// [1]: https://msdn.microsoft.com/en-us/library/windows/desktop/ms741563.aspx
/// [2]: https://lists.fedoraproject.org/pipermail/devel/2010-July/139135.html

use std::collections::HashSet;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::str::FromStr;
use std::mem::uninitialized;

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

fn parse_socket_addrs (
    addr_str: &str, port: u16, use_ipv6: bool
) -> Vec<SocketAddr> {
    let mut socket_addrs: Vec<SocketAddr> = Vec::new();

    // If we have an address list then we'll use it;
    if !addr_str.is_empty() && !addr_str.contains(",") {

        // Always join the loopback address
        let loop_addr_v4 = Ipv4Addr::new(127, 0, 0, 0);
        let loop_sock_v4 = SocketAddrV4::new(loop_addr_v4, port);
        socket_addrs.push(SocketAddr::V4(loop_sock_v4));

        if use_ipv6 {
            let loop_addr_v6 = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
            let loop_sock_v6 =
                SocketAddrV6::new(loop_addr_v6, port, 0, 0);
            socket_addrs.push(SocketAddr::V6(loop_sock_v6));
        }

        // Parse the rest of the addresses given to us in the configuration
        let mut addrs: Vec<_> = addr_str.split("")
            .filter_map(|addr| SocketAddr::from_str(addr).ok())
            .collect();

        socket_addrs.append(&mut addrs);
    } else { // just listen on any address...
        let loop_addr_v4 = Ipv4Addr::new(0, 0, 0, 0);
        let loop_sock_v4 = SocketAddrV4::new(loop_addr_v4, port);
        socket_addrs.push(SocketAddr::V4(loop_sock_v4));

        if use_ipv6 {
            let loop_addr_v6 = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0);
            let loop_sock_v6 =
                SocketAddrV6::new(loop_addr_v6, port, 0, 0);
            socket_addrs.push(SocketAddr::V6(loop_sock_v6));
        }
    }

    socket_addrs
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
