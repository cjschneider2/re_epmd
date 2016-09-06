use std::env;
use std::time::Duration;

use epmd::EpmdConfig;
use usage::display_usage;

pub enum EpmdReq {
    Alive2,
    Port2,
    Names,
    Dump,
    Kill,
    Stop(String)
}

pub enum ParseResponse {
    Ok,
    ShouldExit,
    BadOpt,
    Call(EpmdReq),
}

pub fn parse_args(config: &mut EpmdConfig) -> ParseResponse {

    let mut argv = env::args();
    let argc = argv.len();

    // TODO: No arguments given == `normal` run?
    if argc == 1 {
        return ParseResponse::Ok;
    }

    while let Some(arg) = argv.next() {

        match arg.as_ref() {

            "-d" => config.debug = true,

            "-debug" => config.debug = true,

            "-packet_timeout" => {
                let val: u64 = match argv.next() {
                    Some(s) => s.parse().expect("packet_timeout value err"),
                    None => return ParseResponse::BadOpt
                };
                config.packet_timeout = Duration::new(val, 0);
            },

            "-delay_accept" => {
                let val: usize = match argv.next() {
                    Some(s) => s.parse().expect("delay_accept value err"),
                    None => return ParseResponse::BadOpt
                };
                config.delay_accept = val;
            },

            "-delay_write" => {
                let val: usize = match argv.next() {
                    Some(s) => s.parse().expect("delay_write value err"),
                    None => return ParseResponse::BadOpt
                };
                config.delay_write = val;
            },

            "-daemon" => config.is_daemon = true,

            "-relaxed_command_check" => config.brutal_kill = true,

            "-kill" => {
                if argc == 1 {
                    return ParseResponse::Call(EpmdReq::Kill)
                } else {
                    return ParseResponse::BadOpt
                }
            },

            "-address" => {
                let val: String = match argv.next() {
                    Some(s) => s,
                    None => return ParseResponse::BadOpt
                };
                config.address = val;
            },

            "-port" => {
                let val: usize = match argv.next() {
                    Some(s) => s.parse().expect("port value err"),
                    None => return ParseResponse::BadOpt
                };
                config.port = val as u16;
            },

            "-names" => {
                if argc == 1 {
                    return ParseResponse::Call(EpmdReq::Names)
                } else {
                    return ParseResponse::BadOpt
                }
            },

            "-started" => {
                config.silent = true;
                if argc == 1 {
                    return ParseResponse::Call(EpmdReq::Names)
                } else {
                    return ParseResponse::BadOpt
                }
            },

            "-dump" => {
                if argc == 1 {
                    return ParseResponse::Call(EpmdReq::Dump)
                } else {
                    return ParseResponse::BadOpt
                }
            },

            "-stop" => {
                let name: String = match argv.next() {
                    Some(s) => s,
                    None => return ParseResponse::BadOpt
                };
                return ParseResponse::Call(EpmdReq::Stop(name))
            },

            // TODO: Should only be active if the systemd daemon is available
            // apparently it's hiding under the env_var `HAVE_SYSTEMD_DAEMON`???
            "-systemd" => config.is_systemd = true,

            _ => display_usage(),
        };
    }
    ParseResponse::ShouldExit
}
