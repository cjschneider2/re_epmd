mod constants;
mod usage;
mod empd;
mod enode;

pub use usage::display_usage;

// TODO:
// IMPLEMENT
// FROM: http://www.thegeekstuff.com/2012/02/c-daemon-process
// Daemon Process Design:
// A daemon process can be developed just like any other process but there is
// one thing that differentiates it with any other normal process ie having no
// controlling terminal. This is a major design aspect in creating a daemon
// process.
// This can be achieved by :
//    Create a normal process (Parent process)
//    Create a child process from within the above parent process
//    The process hierarchy at this stage looks like:
//        TERMINAL -> PARENT PROCESS -> CHILD PROCESS
//    Terminate the the parent process.
//    The child process now becomes orphan and is taken over by the init process.
//    Call setsid() fn to run the process in new session and have a new group.
//    After the above step we can say that now this process becomes a daemon
//    process without having a controlling terminal.
//    Change the working directory of the daemon process to root and close
//    stdin, stdout and stderr file descriptors.
//    Let the main logic of daemon process run.
//
// Can look here as well:
// http://www.netzmafia.de/skripten/unix/linux-daemon-howto.html
pub fn run_daemon () {
    // Fork and sure that the child is not a process group leader

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
