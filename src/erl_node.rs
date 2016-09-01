#![allow(dead_code)]

#[derive(PartialEq, Eq, Hash)]
pub struct ErlNode {
    fd: u32, // socket in use
    port: u16, // port number of erlang node
    name: String, // name of the erlang node
    creation: u8, // incremented in the range [1..3] for reused nodes
    node_type: u8, // 77u8 = normal erlang node; 72u8 = hidden (c-node)
    protocol: u8, // 0 = tcp/ipv4
    high_version: u8, // 0 = OTP-R3 erts-4.6.x; 1 = OTP-R4 erts-4.7.x
    low_version: u8, // see above
    extra: String,
}

impl ErlNode {
    pub fn new (
        name: String,
        fd: u32,
        erl_port: u16,
        node_type: u8,
        protocol: u8,
        high_vsn: u8,
        low_vsn: u8,
        extra: String
    ) -> ErlNode {
        ErlNode {
            fd: fd,
            port: 0,
            name: name,
            creation: 1, // NOTE: In the orig. version this was "random"
            node_type: node_type,
            protocol: protocol,
            high_version: high_vsn,
            low_version: low_vsn,
            extra: extra
        }
    }
}
