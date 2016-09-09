use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::net::{TcpListener};

use net2::TcpBuilder;

use constants::IPV6_ONLY;

/// `parse_socket_addrs` assumes that the addresses are given in the forms of:
///    "192.168.1.1"
///    "192.168.1.1, 10.0.0.1"
///    "192.168.1.1 10.0.0.1"
/// or their ipv6 counterparts and sets the listen port to the parameter value.
pub fn parse_socket_addrs (
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
        //socket_addrs.push(get_loopback_address(port, use_ipv6));
        socket_addrs.push(get_any_address(port, use_ipv6));
    }
    socket_addrs
}

/// Creates a list of listening sockets from which we can call select on and
/// check for incoming connections.
// TODO:
//  * Catch errors we throw away through the `let _ = Result<()>` statements.
pub fn create_listen_sockets (in_socks: Vec<SocketAddr>) -> Vec<TcpListener> {
    let sockets =
        in_socks.iter()
        .filter_map(|sock| {
            let builder = match sock.ip() {
                IpAddr::V4(..) => TcpBuilder::new_v4(),
                IpAddr::V6(..) => TcpBuilder::new_v6(),
            };
            match builder {
                Ok(b) => { let _ = b.bind(sock); Some(b) },
                Err(_) => None,
            }
        })
        .map(|b| { let _ = b.reuse_address(true); b })
        .map(|b| { if IPV6_ONLY { let _ = b.only_v6(true); } b })
        .filter_map(|b| b.listen(0).ok())
        .collect();
    sockets
}


pub fn get_loopback_address(port: u16, use_ipv6: bool) -> SocketAddr {
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

pub fn get_address() -> String {
    use std::env::var;
    var("ERL_EPMD_ADDRESS").unwrap_or("".into())
}

pub fn get_port_number() -> u16 {
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
