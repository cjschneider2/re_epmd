#![allow(dead_code)]

pub struct Connection {
    pub open: bool,	// `true` if open
    pub keep: bool,	// Don't close when sent reply
    pub fd: usize, // File descriptor
    pub local_peer: bool, // The peer of this connection is via loopback interface
    pub got: usize,	// # of bytes we have got
    pub want: usize, // Number of bytes we want
    pub buffer: Vec<u8>,	// The remaining buffer
    //TODO: mod_time: time??? // Last activity on this socket
}
