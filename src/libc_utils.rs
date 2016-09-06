use libc::{signal, SIGPIPE, SIG_IGN, FD_ZERO};

use epmd::Epmd;

/// Initializes / Sets the select read mask to zero
pub fn init_select_vars(epmd: &mut Epmd) {
    unsafe {
        FD_ZERO(&mut epmd.orig_read_mask);
    }
    epmd.select_fd_top = 0;
}

/// Ignore the SIGPIPE signal that is raised when we call write
/// twice on a socket closed by the other end.
pub fn ignore_sig_pipe () {
    unsafe {
        signal(SIGPIPE, SIG_IGN);
    }
}
