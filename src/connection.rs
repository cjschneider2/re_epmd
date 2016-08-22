#![allow(dead_code)]

struct Connection {
    open: bool,	// `true` if open
    keep: bool,	// Don't close when sent reply
    fd: usize, // File descriptor
    local_peer: bool, // The peer of this connection is via loopback interface
    got: usize,	// # of bytes we have got
    want: usize, // Number of bytes we want
    buffer: Vec<u8>,	// The remaining buffer
    //TODO: mod_time: time??? // Last activity on this socket
}
