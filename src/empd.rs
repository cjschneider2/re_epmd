#![allow(dead_code)]

use constants::MAX_LISTEN_SOCKETS;

struct Epmd {
    debug: bool,
    silent: bool,
    is_daemon: bool,
    //brutal_kill: bool, // Check if needed
    port: u16,
    packet_timeout: usize,
    delay_accept: usize,
    delay_write: usize,
    max_conn: usize,
    active_conn: usize,
    //select_fd_top: usize, // what is this?
    program_name: String,
    //conn: Connection,
    //nodes: HashMap<Node>,
    listen_fd: [usize; MAX_LISTEN_SOCKETS]
}

impl Epmd {
    pub fn kill () {
        println!("TODO: epmd.kill()");
    }

    pub fn call () {
        println!("TODO: epmd.call()");
    }

    pub fn run () {
        println!("TODO: epmd.call()");
    }
}
