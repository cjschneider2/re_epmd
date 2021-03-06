#![allow(dead_code)]

use std::time::{Instant, Duration};
use std::net::{IpAddr, TcpStream, SocketAddr};
use std::io::{Read, BufReader};
use std::io::{Write};

use constants::INBUF_SIZE;

#[derive(Debug)]
enum Status {
    Idle,
    NeedData,
    NeedResp,
}

#[derive(Debug)]
pub struct Connection {
    pub can_remove: bool,  // The connection is safe to remove from the list
    pub open: bool,	       // `true` if open
    pub keep: bool,	       // Don't close when sent reply
    pub mod_time: Instant, // Last activity on this socket
    pub stream: TcpStream, // TCP connection stream
    peer_addr: SocketAddr, // Remote peer's socket address
    local_peer: bool,      // This connection is via a local/loopback interface
    read_buffer: BufReader<TcpStream>,	// The remaining buffer
    status: Status,
}

impl Connection {
    pub fn new (
        stream: TcpStream,
        peer_addr: SocketAddr,
        timeout: Duration
    ) -> Connection {
        // TODO: Error handling...
        let _ = stream.set_read_timeout(Some(timeout));
        let local_addr = stream.local_addr().unwrap();
        let stream_clone = stream.try_clone().unwrap();
        Connection {
            can_remove: false,
            open: true,
            keep: false,
            stream: stream,
            peer_addr: peer_addr,
            local_peer: is_local_peer(&peer_addr, &local_addr),
            read_buffer: BufReader::new(stream_clone),
            mod_time: Instant::now(),
            status: Status::Idle,
        }
    }

    /// Reads raw data off of the stream
    pub fn read(&mut self) -> Vec<u8> {
        let mut buf = [0; INBUF_SIZE];
        let bytes_recv = match self.read_buffer.read(&mut buf) {
            Ok(size) => size,
            Err(e) => { println!("read() error: {:?}", e);
                        panic!("");
                        0 }
        };
        println!("DEBUG: Received {} bytes.", bytes_recv);

        let mut vec = buf.to_vec();
        vec.truncate(bytes_recv);

        // Error Checking - Correct Length
        let len = if vec.len() >= 2 {
            u16::from_be(vec[0] as u16 | (vec[1] as u16) << 8)
        } else {
            println!("DEBUG: Received packet too short... :(");
            0
        };
        println!("DEBUG: Expected len: {}\nDEBUG: Received len: {}",
                 bytes_recv, len + 2);

        vec
    }

    /// Sends raw data to the remote connection
    pub fn write(&mut self, response: Vec<u8>) {
        let _ = self.stream.write_all(&response);
        let _ = self.stream.flush();
    }

    /// Signals that the Connection should be dropped
    pub fn close(&mut self) {
        self.keep = false;
        self.open = false;
        //let _ = conn.stream.shutdown(Shutdown::Both);
    }

}

/// Function to check to see if the connection comes from a local peer.
/// This function checks the loopback interface and other local addresses.
fn is_local_peer(sock_peer: &SocketAddr, sock_local: &SocketAddr) -> bool {
    // NOTE: IpAddr.is_loopback() is stable since Rust 1.12; Earlier versions
    // need to get the IpAddrV4/6.is_loopback() respectively.
    //let is_loopback  = sock_peer.ip().is_loopback();
    let is_loopback = match sock_peer.ip() {
        IpAddr::V4(ref a) => a.is_loopback(),
        IpAddr::V6(ref a) => a.is_loopback(),
    };
    let is_same_addr = sock_peer.ip() == sock_local.ip();
    (is_loopback || is_same_addr)
}
