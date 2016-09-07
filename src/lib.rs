extern crate libc;
extern crate net2;

mod constants;
mod usage;
mod epmd;
mod connection;
mod erl_node;
mod daemon;
mod parse_args;
mod libc_utils;

pub use usage::display_usage;
pub use parse_args::parse_args;
pub use parse_args::ParseResponse;

pub use epmd::EpmdReq;
pub use epmd::Epmd;
pub use epmd::EpmdConfig;

pub fn run_console (
    epmd: Epmd, config: EpmdConfig, with_request: Option<EpmdReq>
) {
    epmd::run(epmd, config, with_request);
}

#[cfg(unix)]
pub fn run_daemon(epmd: Epmd, config: EpmdConfig) {
    daemon::run_daemon_unix(epmd, config);
}
#[cfg(windows)]
pub fn run_daemon() {
    daemon::run_daemon_win();
}

