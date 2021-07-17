use std::convert::AsRef;

#[derive(Debug, Clone)]
#[repr(C, packed)]
pub struct Ipv4Packet {
    version_and_ihl: u8,
    dscp_and_ecn: u8,
    total_length: u16,
    id: u16,
    flags_and_flagment_offset: u16,
    time_to_live: u8,
    protocol: u8,
    header_checksum: u16,
    src_ip_address: u32,
    dst_ip_address: u32,
    option_and_padding: u32,
    payload: Vec<u8>,
}

pub fn handle(packet: &Ipv4Packet) {}
