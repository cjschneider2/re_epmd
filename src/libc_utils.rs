use libc::{c_int, fd_set, signal, SIGPIPE, SIG_IGN, FD_ZERO, FD_ISSET};

/// Ignore the SIGPIPE signal that is raised when we call write
/// twice on a socket closed by the other end.
pub fn ignore_sig_pipe () {
    unsafe {
        signal(SIGPIPE, SIG_IGN);
    }
}

/// Generates a newly initialized fd_set
pub fn new_fd_set () -> fd_set {
    use std::mem;
    unsafe {
        let mut set = mem::uninitialized();
        FD_ZERO(&mut set);
        set
    }
}

pub fn select_zero_set (set: &mut fd_set) {
    unsafe { FD_ZERO(set); }
}

pub fn select_is_set (fd: c_int, set: &mut fd_set ) -> bool {
    unsafe { FD_ISSET(fd, set) }
}
