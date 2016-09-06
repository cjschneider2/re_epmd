#![allow(dead_code, unused_imports, unused_variables)]

/// Citations:
/// [1]: https://msdn.microsoft.com/en-us/library/windows/desktop/ms741563.aspx
/// [2]: https://lists.fedoraproject.org/pipermail/devel/2010-July/139135.html

use std::str::FromStr;
use std::mem::uninitialized;
use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::net::TcpListener;
use std::net::ToSocketAddrs;
use std::io::Read;
use std::io::ErrorKind;
use std::time::{Duration, Instant};
use std::thread::sleep;

use net2::TcpBuilder;
use net2::TcpListenerExt;
use net2::unix::UnixTcpBuilderExt;

use libc;

use super::run_daemon;
use connection::Connection;
use constants::{MAX_LISTEN_SOCKETS, CLOSE_TIMEOUT, MAX_FILE_DESCRIPTORS};
use constants::{IPV6_ONLY};
use erl_node::ErlNode;
use libc_utils;
use parse_args::EpmdReq;

pub struct EpmdConfig {
    // -- program flags --
    pub debug: bool,
    pub silent: bool,
    pub is_daemon: bool,
    pub is_systemd: bool,
    pub brutal_kill: bool, // Check if needed
    pub use_ipv6: bool,
    // -- extra options --
    pub packet_timeout: Duration,
    pub delay_accept: usize,
    pub delay_write: usize,
    // -- connection properties --
    pub address: String,
    pub port: u16,
}

impl EpmdConfig {
    pub fn new() -> EpmdConfig {
        EpmdConfig {
            // -- program flags --
            debug: false,
            silent: false,
            is_daemon: false,
            is_systemd: false,
            brutal_kill: false,
            use_ipv6: false,
            // -- extra options --
            packet_timeout: Duration::new(CLOSE_TIMEOUT, 0),
            delay_accept: 0,
            delay_write: 0,
            // -- connection properties --
            address: get_address(),
            port: get_port_number(),
            // -- currently unused --
        }
    }
}

pub struct Epmd {
    // TODO: active_conn & listen_fd maybe could be combined into a Vec<fd>
    pub active_conn: usize,
    listen_fd: [i32; MAX_LISTEN_SOCKETS],
    pub max_conn: usize,
    // -- program data --
    //pub nodes: HashSet<ErlNode>,
    //pub connections: Vec<Connection>,
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
            active_conn: 0,
            listen_fd: [-1; MAX_LISTEN_SOCKETS],
            max_conn: MAX_FILE_DESCRIPTORS,
            // -- program data --
            //nodes: HashSet::<ErlNode>::new(),
            //connections: Vec::<Connection>::new(),
            orig_read_mask: unsafe { uninitialized() },
            select_fd_top: 0,
        }
    }

}


pub fn run (mut epmd: Epmd, config: EpmdConfig, with_request: Option<EpmdReq>) {

    /* TODO: systemd related initialization...
    epmd does some querying of the system though systemd if it's available.
    namely using `sd_listen_fds(0)` To get the max # of sockets of the
    system. [2] has a decent way to go about using systemd to find the
    start of the socket counters in this way. This is set elsewhere so I'll
    just ignore this for now. ( the max will just be `MAX_FILE_DESCRIPTORS` )

    IDEA?: maybe put all of the systemd functions into it's own module?
     */

    let addrs = parse_socket_addrs(&config.address, config.port, config.use_ipv6);
    let num_sockets = addrs.len();

    if num_sockets >= MAX_LISTEN_SOCKETS {
        panic!("Cannot listen on more than {} IP Addresses",
               MAX_LISTEN_SOCKETS);
    }

    if cfg!(all(unix)) {
        libc_utils::ignore_sig_pipe();
    }

    // Initialize the number of active file descriptors;
    // `stdin`, `stdout`, & `stderr` are still open.
    epmd.active_conn = 3 + num_sockets;
    epmd.max_conn -= num_sockets;

    // Initialize variables for select()
    libc_utils::init_select_vars(&mut epmd);

    let socks = create_listen_sockets(addrs);

    // Set some socket options
    for sock in socks.iter() {
        // configure non-blocking sockets
        // TODO: I think a better solution would be to block with a timeout
        // which is, i think, what the old code did (with IDLE_TIMEOUT).
        //if let Err(e) = sock.set_nonblocking(true) {
        //    println!("sock.set_nonblocking err:{}", e);
        //}
    }

    // DEBUG
    println!("\nConnected on the following Sockets:");
    for sock in socks.iter() {
        println!("\t{:?}", sock);
    }

    // main event loop
    // the main loop goes something like this:
    //  * Read the select mask too see if there is anything to do
    //  * if there isn't then just busy loop looking for work
    //  * if there is something to do, then set the current time & try to
    //    accept() on all sockets with data until we don't have any one
    //    trying to connect or we've run out of our allowance of
    //    connection sockets.
    //  * For all of our connection objects:
    //    * if we have an open connection then try to `do_read()` on
    //      the socket to communicate with the client.
    //    * or if the connection should be shutdown we'll kill and free the
    //      connection; this can be due to a timeout or the client closing
    //      the connection.
    //let nodes = HashSet::<ErlNode>::new();
    let mut connections = Vec::<Connection>::new();
    loop {
        let now = Instant::now();
        for sock in socks.iter() {
            match sock.accept() {
                Ok((stream, peer_addr)) => {
                    let timeout = Duration::new(0, 500_000_000); // 0.5 sec
                    let conn = Connection::new(stream, peer_addr, timeout);
                    connections.push(conn);
                    println!("Created new connection object");
                }
                Err(err) => {
                    match err.kind() {
                        ErrorKind::Interrupted => {},
                        ErrorKind::WouldBlock => {},
                        ErrorKind::TimedOut => {},
                        error_kind => {
                            // TODO: Recover gracefully...
                            panic!("socket.accept(): {:?}", error_kind);
                        }
                    }
                }
            }
        }

        for mut conn in &mut connections {
            let has_timed_out = conn.mod_time + config.packet_timeout < now;
            if conn.open == true {
                let mesg = conn.do_read();
                let request = parse_request();
                let response = do_request(&mut epmd, request);
                conn.do_write(response);
            } else if conn.keep == false && has_timed_out {
                conn.close();
            }
        }
        connections.retain(|conn| conn.keep);
    }
}

pub fn stop(epmd: &mut Epmd, val: String) {
    val.len();
    unimplemented!();
}

fn parse_request() -> Vec<u8> {
    unimplemented!();
}

fn do_request(epmd: &mut Epmd, request: Vec<u8>) -> Vec<u8> /* response */ {
    vec![0,1,2,3]
}

fn kill (epmd: &mut Epmd) {
    println!("TODO: epmd.kill()");
    unimplemented!();
}

fn call (epmd: &mut Epmd) {
    println!("TODO: epmd.call()");
    unimplemented!();
}

/// Creates a list of listening sockets from which we can call select on and
/// check for incoming connections.
// TODO:
//  * Catch errors we throw away through the `let _ = Result<()>` statements.
fn create_listen_sockets (in_socks: Vec<SocketAddr>) -> Vec<TcpListener> {
    let sockets =
        in_socks.iter()
        .filter_map(|sock| {
            let builder = match sock.ip() {
                IpAddr::V4(..) => TcpBuilder::new_v4(),
                IpAddr::V6(..) => TcpBuilder::new_v6(),
            };
            match builder {
                Ok(b) => { let _ = b.bind(sock); Some(b) },
                Err(e) => None,
            }
        })
        .map(|b| { let _ = b.reuse_address(true); b })
        .map(|b| { if IPV6_ONLY { let _ = b.only_v6(true); } b })
        .filter_map(|b| b.listen(0).ok())
        .collect();
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
