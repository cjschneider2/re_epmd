extern crate re_epmd as epmd;

use epmd::parse_args;

fn main () {
    let mut epmd = epmd::Epmd::new();

    /* TODO: Windows Specific
    In the windows version there is a check of
    `WSAStartup(0x0101, &wsaData)`, to check if the version of
    the socket protocol we want to use is available.
    `wsaData.wVersion != 0x0101` see: [1]
     */

    let should_exit = parse_args(&mut epmd);
    if should_exit { return; }

    /* TODO: Check max file descriptors for system
    See the note @ constants::MAX_FILE_DESCRIPTORS;
     */

    if epmd.is_daemon {
        epmd.run_daemon();
    } else {
        epmd.run();
    }
}
