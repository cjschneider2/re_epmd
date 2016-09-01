use libc;
use std::mem;

use epmd::Epmd;

/// Starts a new epmd daemon process
///
/// The design of making a daemon process is taken from the following resources
/// and is summarized below.
///
/// FROM: [1]
/// Daemon Process Design:
/// A daemon process can be developed just like any other process but there is
/// one thing that differentiates it with any other normal process ie having no
/// controlling terminal. This is a major design aspect in creating a daemon
/// process.
///
/// This can be achieved by :
///  * Create a normal process (Parent process).
///  * Create a child process from within the above parent process.
///    The process hierarchy at this stage looks like:
///        TERMINAL -> PARENT PROCESS -> CHILD PROCESS
///  * Terminate the the parent process.
///  * The child process is now orphaned and is taken over by the init process.
///  * Call setsid() fn to run the process in new session and have a new group.
///    After the above step we can say that now this process becomes a daemon
///    process without having a controlling terminal.
///  * Change the working directory of the daemon process to root and close
///    stdin, stdout and stderr file descriptors.
///    Let the main logic of daemon process run.
///
/// CITATIONS:
/// [1] : http://www.thegeekstuff.com/2012/02/c-daemon-process
/// [2] : http://www.netzmafia.de/skripten/unix/linux-daemon-howto.html
/// [3] : https://www.gnu.org/software/libc/manual/html_node/Termination-Internals.html#Termination-Internals

pub fn run_daemon_unix (mut epmd: Epmd) {

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
    //  * Returns `(pid_t) -1`, i.e. -1:i32, and sets `ERRNO`
    let sid = unsafe { libc::setsid() };
    if  sid < 0 { panic!("epmd: Can't `setsid()`"); }

    // NOTE: This next part comes from the process termination process.
    // FROM: [3]
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
        _  => ()      // continue
    }

    // Move our current working directory to root;
    // to make sure we're not on a mounted file system.
    let chdir = unsafe {
        let path: *const i8 = mem::transmute("/".as_ptr());
        libc::chdir(path)
    };
    if chdir < 0 { panic!("epmd: `chdir()` failed"); }

    // Set the `umask` to `0` which means that this process's file permissions
    // are determined by the system; This need to be changed because the
    // process's file mode creation mask is inherited after a call to `fork()`
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

    // Close the `syslog` with `closelog()`; in case it was opened
    unsafe { libc::closelog() };

    // So, as we've closed all of our file descriptors, and in a single thread,
    // we set `stdin`, `stdout`, and `stderr` to read, write, and write to/from
    // `/dev/null` respectively; in this case the order _is_ important.
    // NOTE:
    //   This is because the POSIX standard file descriptors are defined
    //   as 0, 1, & 2 in for std-in, -out, and -err respectively.
    unsafe {
        let dev_null: *const i8  = mem::transmute("/dev/null".as_ptr());
        libc::open(dev_null, libc::O_RDONLY);
        libc::open(dev_null, libc::O_WRONLY);
        libc::open(dev_null, libc::O_WRONLY);
    }

    // Set the `errno` value to zero just in case it was set by any of
    // `open` system calls.
    unsafe {
        let errno = libc::__errno_location();
        *errno = 0;
    }

    epmd.run();
}

// TODO: Write the windows version of this function
#[allow(dead_code)]
pub fn run_daemon_win () {
    unimplemented!();
}

// TODO: Maybe add an 'other' category if needed?
