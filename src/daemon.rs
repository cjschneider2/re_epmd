use libc;
use std::mem;

/// Starts a new epmd daemon process
///
/// The design of making a daemon process is taken from the following resources
/// and is summarized below.
///
/// FROM: http://www.thegeekstuff.com/2012/02/c-daemon-process
/// Daemon Process Design:
/// A daemon process can be developed just like any other process but there is
/// one thing that differentiates it with any other normal process ie having no
/// controlling terminal. This is a major design aspect in creating a daemon
/// process.
/// This can be achieved by :
///    Create a normal process (Parent process)
///    Create a child process from within the above parent process
///    The process hierarchy at this stage looks like:
///        TERMINAL -> PARENT PROCESS -> CHILD PROCESS
///    Terminate the the parent process.
///    The child process now becomes orphan and is taken over by the init process.
///    Call setsid() fn to run the process in new session and have a new group.
///    After the above step we can say that now this process becomes a daemon
///    process without having a controlling terminal.
///    Change the working directory of the daemon process to root and close
///    stdin, stdout and stderr file descriptors.
///    Let the main logic of daemon process run.
///
/// You can look here as well for some other information.
/// http://www.netzmafia.de/skripten/unix/linux-daemon-howto.html
pub fn run_daemon_unix () {
    // create the parent process
    // NOTE: For the `fork()` call:
    //  * The child has a return value of 0
    //  * The parent has a return value == the pid of the child
    //  * In case of error the return value == -1
    let   child_pid = unsafe { libc::fork() };
    match child_pid {
        -1 => panic!("Erlang mapper daemon can't fork"),
         0 => return, // Parent should exit
         _ => () // continue
    }

    // Become the session leader
    // NOTE: for the `setsid()` call:
    //  * Returns new process group ID if successful
    //  * Returns `(pid_t)-1`, i.e. -1:i32, and sets `ERRNO`
    let sid = unsafe { libc::setsid() };
    if  sid < 0 { panic!("epmd: Can't `setsid()`"); }

    // NOTE: This next part comes from the process termination process.
    // FROM: https://www.gnu.org/software/libc/manual/html_node/Termination-Internals.html#Termination-Internals
    //   "If the process is a session leader that has a controlling terminal,
    //    then a SIGHUP signal is sent to each process in the foreground job,
    //    and the controlling terminal is disassociated from that session.
    // i.e. We want to ignore the `SIGHUP` signal when `our terminal` closes
    unsafe { libc::signal(libc::SIGHUP, libc::SIG_IGN) };

    // We don't want to actually be the session leader so fork again
    let   child_pid = unsafe { libc::fork() };
    match child_pid {
        -1 => panic!("Erlang mapper daemon can't complete second fork"),
        0  => return, // Parent should exit
        _  => () // continue
    }

    // Move our current working directory to root;
    // to make sure we're not on a mounted file system.
    let chdir = unsafe {
        let path: *const i8 = mem::transmute("/".as_ptr());
        libc::chdir(path)
    };
    if chdir < 0 { panic!("epmd: `chdir()` failed"); }

    // Clear all file rights to this process as the process's file mode
    // creation mask is inherited after a call to `fork()`
    unsafe { libc::umask(0); }

    // Close all open file handles;
    // this includes the default ones for `stdin` etc..
    unsafe {
        let mut limit = mem::uninitialized();
        let _ = libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit);
        for fd in 0..limit.rlim_max {
            // TODO: CHECK SAFETY of type cast from u64 -> i32 here
            libc::close(fd as i32);
        }
    }

    // TODO: Close `syslog` on linux with `closelog()`

    // TODO: Figure out why the following was done in the c-code
    // open("/dev/null", O_RDONLY); // order important?!
    // open("/dev/null", O_WRONLY);
    // open("/dev/null", O_WRONLY);

    // TODO: orig: `errno = 0;`

    // TODO: orig: `run(g);`
}

// TODO: Write the windows version of this function
#[allow(dead_code)]
pub fn run_daemon_win () {
    unimplemented!();
}

// TODO: Maybe add an 'other' category if needed?
