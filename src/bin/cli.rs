extern crate re_epmd as epmd;

use epmd::{parse_args, display_usage};
use epmd::{run_console, run_daemon};
use epmd::{ParseResponse as PR, EpmdReq as REQ};

fn main () {
    let epmd = epmd::Epmd::new();
    let mut config = epmd::EpmdConfig::new();

    if cfg!(target_os = "windows") {
        check_wsa_version();
    }

    let mut with_request: Option<REQ> = None;
    match parse_args(&mut config) {
        PR::Ok         => {},
        PR::ShouldExit => { return; },
        PR::BadOpt     => { display_usage(); return; },
        PR::Call(req)  => { with_request = Some(req)}
    }

    /* TODO: Check max file descriptors for system
    See the note @ constants::MAX_FILE_DESCRIPTORS;
     */

    if config.is_daemon {
        run_daemon(epmd, config);
    } else {
        run_console(epmd, config, with_request);
    }
}

/// In the windows version there is a check of `WSAStartup(0x0101, &wsaData)`,
/// to check if the version of the socket protocol we want to use is available.
/// i.e.: `wsaData.wVersion != 0x0101` see: [1]
#[cfg(target_os = "windows")]
fn check_wsa_version() -> bool { unimplemented!(); }
#[cfg(not(target_os = "windows"))]
fn check_wsa_version() -> bool { true }
