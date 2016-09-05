extern crate libc;
extern crate net2;

mod constants;
mod usage;
mod epmd;
mod connection;
mod erl_node;
mod daemon;
mod parse_args;

pub use usage::display_usage;
pub use parse_args::parse_args;
pub use epmd::Epmd;

#[cfg(unix)]
pub fn run_daemon(epmd: Epmd) {
    daemon::run_daemon_unix(epmd);
}
#[cfg(windows)]
pub fn run_daemon() {
    daemon::run_daemon_win();
}

