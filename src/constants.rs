#![allow(dead_code)]

// from /erts/epmd/epmd.mk

// Server Defines
// --------------

// EPMD port number
//   4365 - Version 4.2                 TCP
//   4366 - Version 4.3                 TCP
//   4368 - Version 4.4.0 - 4.6.2       TCP
//   4369 - Version 4.6.3 - 4.7.4       TCP/UDP
pub const EPMD_PORT_NUMBER: u16 = 4369;

// Client Defines
// --------------

// Node type:
//    72 = R3 hidden node
//    77 = R3 erlang node
//   104 = R4 hidden node
//   109 = R4 erlang node
//   (110 = R6 nodes (explicit flags for differences between nodes))
//
// What epmd has been told, differs very much between versions, both
// 111 and 110 seems to have been used to tell epmd, while 
// the actual nodetypes has still been 104 and 109. 
// EPMD does not care about this, why we move back to using
// the correct tag (an 'n') for all nodes.
//

const EPMD_NODE_TYPE: u16  = 110;

// Lowest/Highest supported version of the distribution protocol:
//   0 = R3
//   1 = R4
//   2 = R5      ?????
//   3 = R5C
//   4 = R6 (development)
//   5 = R6
// There was no protocol change in release R5, so we didn't need to raise
// the version number. But now that R5A is released, it's best to keep it
// this way.
// The number was inadvertently raised for R5C, so we increase it again
// for R6.
// Distribution version 4 means a) distributed monitor and b) larger references
// in the distribution format. 
// In format 5, nodes can explicitly tell each other which of the above
// mentioned capabilities they can handle.
// Distribution format 5 contains the new md5 based handshake.

const EPMD_DIST_LOW:  u16 = 5;
const EPMD_DIST_HIGH: u16 = 5;

// from /erts/epmd/src/epmd.h

/* Definitions of message codes */

/* Registration and queries */
const EPMD_ALIVE2_RESP: u8 = 121; // 'y'
const EPMD_ALIVE2_REQ: u8  = 120; // 'x'
const EPMD_PORT2_RESP: u8  = 119; // 'w'
const EPMD_PORT2_REQ: u8   = 122; // 'z'
const EPMD_NAMES_REQ: u8   = 110; // 'n'

/* Interactive client command codes */
const EPMD_DUMP_REQ: u8 = 100; // 'd'
const EPMD_KILL_REQ: u8 = 107; // 'k'
const EPMD_STOP_REQ: u8 = 115; // 's'

// from /erts/epmd/src/epmd_int.h
// `-> (at least selection from here...)

// If no activity we let select() return every IDLE_TIMEOUT second
// A file descriptor that has been idle for CLOSE_TIMEOUT seconds and
// isn't an ALIVE socket has probably hanged and should be closed
pub const IDLE_TIMEOUT:  i64 = 5;
pub const CLOSE_TIMEOUT: u64 = 60;

// We save the name of nodes that are unregistered. If a new
// node register the name we want to increment the "creation",
// a constant 1..3. But we put an limit to this saving to keep
// the lookup fast and not to leak memory.
const MAX_UNREG_COUNT: usize       = 1000;
const DEBUG_MAX_UNREG_COUNT: usize = 5;

// Maximum length of a node name == atom name is 255 characters;
// encoded in UTF-8 this gives a max of (255*4) or 1020 bytes.
const MAX_SYM_LEN: usize = 1020;
// NOTE: Since this is just the name as an atom, which is utf8, then
// we can just set this to the max atom length in Erlang, which is:
const MAX_ATOM_LEN: usize = 255;

pub const MAX_LISTEN_SOCKETS: usize = 16;

/* TODO/NOTE: Decide on the maximum number of socket connections
This is apparently a strangely hard to define parameter between different
platforms... This is set once as less than `libc::FD_SETSIZE` if it exists,
or `MAX_FILE` which is defined in the constants...
so, MAX_FILE is defined here to be 2048, which the default for FD_SETSIZE
can, theoretically, be user defined but is set in the Unix's to 1024, which
is probably the limit that most people will use. We can go looking for a
value with ENV_VAR && with libc::FD_SETSIZE, but the default of 1024 should
be fine... I hope...
 */
//const MAX_FILE: usize = 2048;
pub const MAX_FILE_DESCRIPTORS: usize = 1024;

// Largest request: ALIVE2_REQ
//     2 + 13 + 2*MAXSYMLEN
// Largest response: PORT2_RESP
//     2 + 14 + 2*MAXSYMLEN
// That is, 3*MAXSYMLEN should be large enough
pub const INBUF_SIZE:  usize = (3 * MAX_SYM_LEN);
const OUTBUF_SIZE: usize = (3 * MAX_SYM_LEN);

// sets the sockets to only use ipv6
// TODO: Have this option as a feature / configuration option
pub const IPV6_ONLY: bool = false;
