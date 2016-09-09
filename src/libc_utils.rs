use std::ptr;
use std::mem;
use std::io::Error;
use std::io::Result;

use libc::{
    self,
    c_int,
    fd_set, select as __select,
    FD_ZERO, FD_ISSET, FD_SET, FD_CLR,
    signal,
    timeval, clock_gettime, timespec,
    CLOCK_MONOTONIC, SIGPIPE, SIG_IGN,
};

use constants::IDLE_TIMEOUT;

/// Generates a random number between 1 and 3
pub fn rand_1_3() -> u16 {
    unsafe {
        let mut time = timespec { tv_sec: IDLE_TIMEOUT, tv_nsec: 0 };
        clock_gettime(CLOCK_MONOTONIC, &mut time);
        ( time.tv_sec % 3 ) as u16 + 1
    }
}

/// Ignore the SIGPIPE signal that is raised when we call write
/// twice on a socket closed by the other end.
pub fn ignore_sig_pipe () {
    unsafe {
        signal(SIGPIPE, SIG_IGN);
    }
}

/// Generates a newly initialized fd_set
pub fn new_fd_set () -> fd_set {
    unsafe {
        let mut set = mem::uninitialized();
        FD_ZERO(&mut set);
        set
    }
}

pub fn select_zero_set (set: &mut fd_set) {
    unsafe { FD_ZERO(set); }
}

pub fn select_is_set ( set: &mut fd_set, fd: c_int) -> bool {
    unsafe { FD_ISSET(fd, set) }
}

pub fn select_fd_set( set: &mut fd_set, fd: c_int) {
    unsafe { FD_SET(fd, set); }
}

pub fn select_fd_clr( set: &mut fd_set, fd: c_int) {
    unsafe { FD_CLR(fd, set); }
}

pub fn select ( set: &mut fd_set, fd_top: c_int ) -> Result<usize> {
    let mut timeout = timeval { tv_sec: IDLE_TIMEOUT, tv_usec: 0 };
    let events = unsafe {
        __select(
            fd_top,
            set,             /* read  fds */
            ptr::null_mut(), /* write fds */
            ptr::null_mut(), /* error fds */
            &mut timeout)
    };
    if events < 0 {
        let e = Error::last_os_error();
        match e.raw_os_error().unwrap() {
            // Just because all of these aren't defined by ErrorKind...
            libc::EINTR  => { /* interrupted; this is okay */ },
            libc::EINVAL => { /* timeout;     this is okay */ },
            _ => {
                // Can also return the following:
                // EBADF: invalid fd
                // EINVAL: can be:
                //  - invalid timeout specified
                //  - `fd_limit` is less than 0 or greater than `FD_SETSIZE`
                //  - One of the fds refers to a STREAM or Multiplexer
                return Err(e)
            },
        }
    }
    Ok(events as usize)
}
