extern crate libc;

mod constants;
mod usage;
mod epmd;
mod enode;
mod daemon;

pub use usage::display_usage;

#[cfg(unix)]
pub fn run_daemon() {
    daemon::run_daemon_unix();
}
#[cfg(windows)]
pub fn run_daemon() {
    daemon::run_daemon_win();
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

pub fn check_relaxed() -> bool {
    use std::env::var;
    match var("ERL_EPMD_RELAXED_COMMAND_CHECK") {
        Ok(_)  => true,
        Err(_) => false
    }
}
