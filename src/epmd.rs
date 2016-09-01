#![allow(dead_code)]

/// Citations:
/// [1]: https://msdn.microsoft.com/en-us/library/windows/desktop/ms741563.aspx
/// [2]: https://lists.fedoraproject.org/pipermail/devel/2010-July/139135.html

use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::str::FromStr;

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
    max_conn: usize,
    // TODO: active_conn & listen_fd maybe could be combined into a Vec<fd>
    active_conn: usize,
    listen_fd: [i32; MAX_LISTEN_SOCKETS],
    // -- program data --
    nodes: HashSet<ErlNode>,
    conn: Vec<Connection>,
    // -- currently unused --
    //select_fd_top: usize, // what is this?
}

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

        let mut socket_addrs: Vec<SocketAddr> = Vec::new();

        /* TODO: systemd related initialization...
        epmd does some querying of the system though systemd if it's available.
        namely using `sd_listen_fds(0)` To get the max # of sockets of the
        system. [2] has a decent way to go about using systemd to find the
        start of the socket counters in this way. This is set elsewhere so I'll
        just ignore this for now. ( the max will just be `MAX_FILE_DESCRIPTORS` )
        */

        // Initialize listening port
        if !self.address.is_empty() && !self.address.contains(",") {
            // Always join the loopback address
            let loop_addr_v4 = Ipv4Addr::new(127, 0, 0, 0);
            let loop_sock_v4 = SocketAddrV4::new(loop_addr_v4, self.port);
            socket_addrs.push(SocketAddr::V4(loop_sock_v4));

            if self.use_ipv6 {
                let loop_addr_v6 = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
                let loop_sock_v6 =
                    SocketAddrV6::new(loop_addr_v6, self.port, 0, 0);
                socket_addrs.push(SocketAddr::V6(loop_sock_v6));
            }

            // Parse the rest of the addresses given to us in the configuration
            let addrs: Vec<_> = self.address.split("")
                .filter_map(|addr| SocketAddr::from_str(addr).ok())
                .collect();


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
