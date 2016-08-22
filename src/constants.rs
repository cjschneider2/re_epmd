#![allow(dead_code)]

// from /erts/epmd/epmd.mk

// Server Defines
// --------------

// EPMD port number
//   4365 - Version 4.2                 TCP
//   4366 - Version 4.3                 TCP
//   4368 - Version 4.4.0 - 4.6.2       TCP
//   4369 - Version 4.6.3 - 4.7.4       TCP/UDP
const EPMD_PORT_NUMBER: u16 = 4369;

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

const EPMD_DIST_LOW: u16  = 5;
const EPMD_DIST_HIGH: u16 = 5;

// from /erts/epmd/src/epmd.h

/* Definitions of message codes */

/* Registration and queries */
const EPMD_ALIVE2_RESP:u8 = 121; // 'y'
const EPMD_ALIVE2_REQ:u8  = 120; // 'x'
const EPMD_PORT2_RESP:u8  = 119; // 'w'
const EPMD_PORT2_REQ:u8   = 122; // 'z'
const EPMD_NAMES_REQ:u8   = 110; // 'n'

/* Interactive client command codes */
const EPMD_DUMP_REQ:u8 = 100; // 'd'
const EPMD_KILL_REQ:u8 = 107; // 'k'
const EPMD_STOP_REQ:u8 = 115; // 's'

// from /erts/epmd/src/epmd_int.h
// `-> (at least selection from here...)

// If no activity we let select() return every IDLE_TIMEOUT second
// A file descriptor that has been idle for CLOSE_TIMEOUT seconds and
// isn't an ALIVE socket has probably hanged and should be closed
const IDLE_TIMEOUT:u8  = 5;
const CLOSE_TIMEOUT:u8 = 60;

// We save the name of nodes that are unregistered. If a new
// node register the name we want to increment the "creation",
// a constant 1..3. But we put an limit to this saving to keep
// the lookup fast and not to leak memory.
const MAX_UNREG_COUNT:usize       = 1000;
const DEBUG_MAX_UNREG_COUNT:usize = 5;

// Maximum length of a node name == atom name
// 255 characters; UTF-8 encoded -> max 255*4
const MAXSYMLEN:usize = (255*4);

const MAX_LISTEN_SOCKETS:usize = 16;

// Largest request: ALIVE2_REQ
//  2 + 13 + 2*MAXSYMLEN
// Largest response: PORT2_RESP
//  2 + 14 + 2*MAXSYMLEN
// That is, 3*MAXSYMLEN should be large enough
const INBUF_SIZE:usize  = (3*MAXSYMLEN);
const OUTBUF_SIZE:usize = (3*MAXSYMLEN);
