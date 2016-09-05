use std::env;

use epmd::Epmd;
use usage::display_usage;

pub fn parse_args(epmd: &mut Epmd) -> bool /* should_exit */ {

    let mut should_exit: bool = false;

    let mut argv = env::args();
    let argc = argv.len();

    // TODO: No arguments given == `normal` run?
    if argc == 1 {
        return false;
    }

    'arg: loop {
        let arg = match argv.next() {
            Some(string) => string,
            None => break 'arg
        };

        match arg.as_ref() {

            "-d" => epmd.debug = true,

            "-debug" => epmd.debug = true,

            "-packet_timeout" => {
                let val: usize = match argv.next() {
                    Some(s) => s.parse().expect("packet_timeout value err"),
                    None => {
                        display_usage();
                        should_exit = true;
                        break 'arg
                    }
                };
                epmd.packet_timeout = val;
            },

            "-delay_accept" => {
                let val: usize = match argv.next() {
                    Some(s) => s.parse().expect("delay_accept value err"),
                    None => {
                        display_usage();
                        should_exit = true;
                        break 'arg
                    }
                };
                epmd.delay_accept = val;
            },

            "-delay_write" => {
                let val: usize = match argv.next() {
                    Some(s) => s.parse().expect("delay_write value err"),
                    None => {
                        display_usage();
                        break 'arg
                    }
                };
                epmd.delay_write = val;
            },

            "-daemon" => epmd.is_daemon = true,

            "-relaxed_command_check" => epmd.brutal_kill = true,

            "-kill" => {
                if argc == 1 {
                    epmd.kill();
                } else {
                    display_usage();
                }
                should_exit = true;
                break 'arg
            },

            "-address" => {
                let val: String = match argv.next() {
                    Some(s) => s,
                    None => {
                        display_usage();
                        should_exit = true;
                        break 'arg
                    }
                };
                epmd.address = val;
            },

            "-port" => {
                let val: usize = match argv.next() {
                    Some(s) => s.parse().expect("port value err"),
                    None => {
                        display_usage();
                        should_exit = true;
                        break 'arg
                    }
                };
                epmd.port = val as u16;
                // TODO: Check if there is another argument and stick that
                // in the `epmd.port` value
            },

            "-names" => {
                if argc == 1 {
                    unimplemented!();
                    // epmd.call(EPMD_NAMES_REQ)
                } else {
                    display_usage();
                }
                should_exit = true;
                break 'arg
            },

            "-started" => {
                epmd.silent = true;
                if argc == 1 {
                    unimplemented!();
                    // epmd.call(EPMD_NAMES_REQ)
                } else {
                    display_usage();
                }
                should_exit = true;
            },

            "-dump" => {
                if argc == 1 {
                    unimplemented!();
                    // epmd.call(EPMD_DUMP_REQ)
                } else {
                    display_usage();
                }
                should_exit = true;
                break 'arg
            },

            "-stop" => {
                let val: String = match argv.next() {
                    Some(s) => s,
                    None => {
                        display_usage();
                        should_exit = true;
                        break 'arg
                    }
                };
                epmd.stop(val); // (orig: stop_cli(g, argv[1]))
                should_exit = true;
                break 'arg
            },

            // TODO: Should only be active if the systemd daemon is available
            // apparently it's hiding under the env_var `HAVE_SYSTEMD_DAEMON`???
            "-systemd" => epmd.is_systemd = true,

            _ => display_usage(),
        };
    }
    should_exit
}
