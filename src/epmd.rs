#![allow(dead_code, unused_imports, unused_variables)]

/// Citations:
/// [1]: https://msdn.microsoft.com/en-us/library/windows/desktop/ms741563.aspx
/// [2]: https://lists.fedoraproject.org/pipermail/devel/2010-July/139135.html

use std::io::{Read, Result, ErrorKind, Error};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::net::{TcpListener, ToSocketAddrs};
use std::time::{Duration, Instant};
use std::collections::HashSet;
use std::mem::zeroed;
use std::thread::sleep;
use std::str::from_utf8;

use net2::TcpBuilder;
use net2::TcpListenerExt;
use net2::unix::UnixTcpBuilderExt;

use libc;

use super::run_daemon;
use connection::Connection;
use constants::{MAX_LISTEN_SOCKETS, CLOSE_TIMEOUT, MAX_FILE_DESCRIPTORS};
use constants::{IPV6_ONLY, ALIVE2_RESP, PORT2_RESP};
use erl_node::ErlNode;
use libc_utils;

#[derive(Debug)]
pub enum EpmdReq {
    None,
    // port, type, protocol, high_ver, low_ver, name, extra
    Alive2(u16, u8, u8, u16, u16, String, Vec<u8>),
    Port2(String), // Name
    Names,
    Dump,
    Kill,
    Stop(String) // Name
}

#[derive(Debug)]
pub enum EpmdResp {
    None,
    Alive2(u8, u16), // Result, Creation
    Port2Err(u8),    // just result is given if error.
    // result, port, type, protocol, high_ver, low_ver, name, extra
    Port2Ok(u8, u16, u8, u8, u16, u16, String, Vec<u8>),
    Names(u32, String),
    Dump(u32, String),
    KillErr(String), // TODO: Check to see if this is used in epmd
    KillOk(String),  // "OK" is sent if successful
    StopErr(String), // "NOEXIST" is sent if node doesn't exist
    StopOk(String),  // "STOPPED" is sent if node is removed
}

struct Select {
    pub fd_top: libc::c_int,     // Max file descriptor value + 1
    pub fd_set: libc::fd_set, // Select read-mask
}

impl Select {
    fn new () -> Select {
        Select {
            fd_top: 0,
            fd_set: libc_utils::new_fd_set(),
        }
    }

    fn zero_set(&mut self) {
        libc_utils::select_zero_set(&mut self.fd_set);
    }

    fn check(&mut self, fd: libc::c_int) -> bool {
        libc_utils::select_is_set(&mut self.fd_set, fd)
    }

    fn select(&self, mut set: libc::fd_set) -> Result<usize> {
        libc_utils::select(&mut set, self.fd_top, )
    }

    fn set_fd(&mut self, sock: &TcpListener) {
        let fd = get_raw_fd(sock) as libc::c_int;
        libc_utils::select_fd_set(&mut self.fd_set, fd);
        if fd >= self.fd_top {
            self.fd_top = fd + 1;
        }
    }
}

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
    pub active_conn: usize,
    pub max_conn: usize,
    pub nodes: HashSet<ErlNode>,
}


#[cfg(target_pointer_width = "32")]
const POINTER_BITS:usize = 32;
#[cfg(target_pointer_width = "64")]
const POINTER_BITS:usize = 64;

impl Epmd {
    pub fn new () -> Epmd {
        Epmd {
            active_conn: 0,
            max_conn: MAX_FILE_DESCRIPTORS,
            nodes: HashSet::<ErlNode>::new(),
        }
    }
}


pub fn run (
    mut epmd: Epmd,
    config: EpmdConfig,
    with_request: Option<EpmdReq>
) {
    println!("");
    /* TODO: systemd related initialization...
    epmd does some querying of the system though systemd if it's available.
    namely using `sd_listen_fds(0)` To get the max # of sockets of the
    system. [2] has a decent way to go about using systemd to find the
    start of the socket counters in this way. This is set elsewhere so I'll
    just ignore this for now. ( the max will just be `MAX_FILE_DESCRIPTORS` )

    IDEA?: maybe put all of the systemd functions into it's own module?
     */

    let addrs =
        parse_socket_addrs(
            &config.address,
            config.port,
            config.use_ipv6);

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

    let listeners = create_listen_sockets(addrs);

    // configure sockets for select()
    let mut select = Select::new();
    for sock in listeners.iter() {
        println!("creating set_fd for {:?}", sock);
        select.set_fd(sock);
        sock.set_nonblocking(true).expect("sock.set_nonblocking()");
    }

    // DEBUG
    println!("Connected on the following Sockets:");
    for sock in listeners.iter() {
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
        let read_mask = select.fd_set.clone();

        println!("before do_select()");

        let events = select.select(read_mask).expect("Select()");
        if events == 0 {
            select.zero_set();
        }

        println!("got {} events", events);

        for sock in listeners.iter() {
            let fd = get_raw_fd(sock);
            if select.check(fd) {
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
        }

        for mut conn in &mut connections {
            let has_timed_out = conn.mod_time + config.packet_timeout < now;
            if conn.open == true {
                let mesg = conn.do_read();
                let request = parse_request(mesg);
                println!("Got request: {:?}", request);
                let response = do_request(&mut epmd, request);
                match response {
                    EpmdResp::None => {},
                    _ => {
                        let resp_data = serialize_response(response);
                        conn.do_write(resp_data);
                    }

                }
            } else if !conn.keep && has_timed_out {
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

fn serialize_response(resp: EpmdResp) -> Vec<u8> {
    let ser_u16 = |n: u16| -> [u8; 2] {
        let be = u16::to_be(n);
        [(be & 0xFF) as u8, (be >> 8 & 0xFF) as u8]
    };
    let ser_u32 = |n: u32| -> [u8; 4] {
        let be = u32::to_be(n);
        [(be & 0xFF) as u8,
         (be >> 8 & 0xFF) as u8,
         (be >> 16 & 0xFF) as u8,
         (be >> 24 & 0xFF) as u8]
    };
    match resp {
        EpmdResp::None => vec![],
        EpmdResp::Alive2(result, creation) => {
            let c = ser_u16(creation);
            vec![ALIVE2_RESP, result, c[0], c[1]]
        }
        EpmdResp::Port2Err(errno) => {
            vec![PORT2_RESP, errno]
        }
        EpmdResp::Port2Ok(res, port, n_type, proto, hver, lver, name, ext) => {
            let pt    = ser_u16(port);
            let hv    = ser_u16(hver);
            let lv    = ser_u16(lver);
            let n_len = ser_u16(name.len() as u16);
            let e_len = ser_u16(ext.len() as u16);
            let mut resp = vec![
                PORT2_RESP, res,
                pt[0], pt[1],
                n_type, proto,
                hv[0], hv[1],
                lv[0], lv[1],
                n_len[0], n_len[1]
            ];
            resp.extend_from_slice(&name.into_bytes());
            resp.push(e_len[0]);
            resp.push(e_len[1]);
            resp.extend_from_slice(&ext);
            resp
        }
        EpmdResp::Names(epmd_port, name_list) |
        EpmdResp::Dump(epmd_port, name_list) => {
            let ep = ser_u32(epmd_port);
            let mut resp = vec![ep[0], ep[1], ep[2], ep[3]];
            resp.extend_from_slice(&name_list.into_bytes());
            resp
        }
        EpmdResp::KillErr(s) => { vec![] },
        EpmdResp::KillOk(s)  => { vec![79, 75] /* "OK" */ },
        EpmdResp::StopErr(s) => { vec![78, 79, 69, 88, 73, 83, 84] }, //"NOEXIST"
        EpmdResp::StopOk(s)  => { vec![83, 84, 79, 80, 80, 69, 68] }, //"STOPPED"
    }
}

fn parse_request(mesg: Vec<u8>) -> EpmdReq {

    // parser's little helper function :)
    let parse_u16 = |a:u8, b:u8| -> u16 {
        u16::from_be(a as u16 | (b as u16) << 8)
    };

    if mesg.len() < 3 { return EpmdReq::None; }

    let (len_v, data ) = mesg.split_at(2);
    let (req, data )   = data.split_at(1);

    let len = parse_u16(len_v[0], len_v[1]);

    match req[0] {
        120 => {
            if data.len() < 12 { /* min data: 2+1+1+2+2+2+0+2+0 = 12 bytes */
                EpmdReq::None
            } else {
                // start fixed-len header
                let port      = parse_u16(data[0], data[1]);
                let node_type = data[2];
                let protocol  = data[3];
                let high_ver  = parse_u16(data[4], data[5]);
                let low_ver   = parse_u16(data[6], data[7]);
                // end fixed-len header
                let (_done, data) = data.split_at(8);
                let (len,   data) = data.split_at(2);
                // parse name with bound check
                let _nlen    = parse_u16(data[0], data[1]);
                let name_len = if _nlen as usize > (data.len() - 2) {
                    data.len() as u16 - 2
                } else {
                    _nlen
                };
                let (_name, data) = data.split_at(name_len as usize);
                let name = from_utf8(_name).unwrap_or("INVALID_NAME");
                // parse Extra
                let (ex_len, extra) = data.split_at(2);

                EpmdReq::Alive2(port, node_type, protocol, high_ver,
                                low_ver, name.to_string(), extra.to_owned())
            }
        }
        122 => {
            match from_utf8(data) {
                Ok(name) => EpmdReq::Port2(name.to_string()),
                Err(_)   => EpmdReq::None
            }
        }
        110 => EpmdReq::Names,
        100 => EpmdReq::Dump,
        107 => EpmdReq::Kill,
        115 => {
            match from_utf8(data) {
                Ok(name) => EpmdReq::Stop(name.to_string()),
                Err(_)   => EpmdReq::None
            }
        }
        _ => EpmdReq::None
    }
}

fn do_request(epmd: &mut Epmd, req: EpmdReq) -> EpmdResp {
    match req {
        EpmdReq::None => EpmdResp::None,
        EpmdReq::Alive2(port, n_type, protocol, h_ver, l_ver, name, extra) => {
            EpmdResp::None
        }
        EpmdReq::Port2(name) => {
            EpmdResp::None
        },
        EpmdReq::Names => {
            EpmdResp::None
        },
        EpmdReq::Dump => {
            EpmdResp::None
        },
        EpmdReq::Kill => {
            EpmdResp::None
        },
        EpmdReq::Stop(name) => {
            EpmdResp::None
        },
    }
}

fn kill (epmd: &mut Epmd) {
    println!("TODO: epmd.kill()");
    unimplemented!();
}

fn call (epmd: &mut Epmd) {
    println!("TODO: epmd.call()");
    unimplemented!();
}

#[cfg(target_os = "windows")]
fn get_raw_fd(listener: TcpListener) -> libc::c_int {
    listener.as_raw_socket() as libc::c_int
}
#[cfg(any(unix))]
fn get_raw_fd(listener: &TcpListener) -> libc::c_int {
    use std::os::unix::io::AsRawFd;
    listener.as_raw_fd() as libc::c_int
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
