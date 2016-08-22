#![allow(dead_code)]

struct Enode {
    fd: u32, // socket in use
    port: u16, // port number of erlang node
    symname: String, // name of the erlang node
    //creation: u16, // started as a random port number
    node_type: u8, // 77u8 = normal erlang node; 72u8 = hidden (c-node)
    protocol: u8, // 0 = tcp/ipv4
    high_version: u8, // 0 = OTP-R3 erts-4.6.x; 1 = OTP-R4 erts-4.7.x
    low_version: u8, // see above
    extra: String,
}
