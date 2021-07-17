use std::convert::TryInto;
use std::fmt;
use std::num::ParseIntError;

use crate::ipv4;
use crate::ipv4::Ipv4Packet;
use crate::packet::Packet;

pub const MAC_LENGTH: usize = 6;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress([u8; MAC_LENGTH]);

pub const MAC_ANY: MacAddress = MacAddress([0; MAC_LENGTH]);
pub const MAC_BROADCAST: MacAddress = MacAddress([0xff; MAC_LENGTH]);

impl MacAddress {
    pub fn from_str(address_str: &str) -> Result<Self, ParseIntError> {
        let mac_address: [u8; MAC_LENGTH] = address_str
            .split(':')
            .map(|s| u8::from_str_radix(s, 16))
            .collect::<Result<Vec<u8>, ParseIntError>>()?
            .try_into()
            .unwrap_or_else(|v: Vec<u8>| {
                panic!(
                    "Expected a Vec of length {}, but it was {}",
                    MAC_LENGTH,
                    v.len()
                )
            });
        Ok(MacAddress(mac_address))
    }
}

impl Default for MacAddress {
    fn default() -> MacAddress {
        MacAddress([0; MAC_LENGTH])
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:X?}:{:X?}:{:X?}:{:X?}:{:X?}:{:X?}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[repr(u16)]
pub enum PacketType {
    Ipv4 = 0x0800,
    Arp = 0x0806,
    Rarp = 0x8035,
    AppleTalk = 0x809b,
    Ieee802 = 0x8100,
    Ipx = 0x8137,
    Ipv6 = 0x86dd,
}

impl fmt::Display for PacketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PacketType::Ipv4 => "IPv4",
                PacketType::Arp => "ARP",
                PacketType::Rarp => "RARP",
                PacketType::AppleTalk => "AppleTalk",
                PacketType::Ieee802 => "IEEE802",
                PacketType::Ipx => "IPX",
                PacketType::Ipv6 => "IPv6",
            }
        )
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct EthernetPacket {
    dst_mac: MacAddress,
    src_mac: MacAddress,
    packet_type: PacketType,
    payload: Vec<u8>,
}

impl EthernetPacket {
    pub fn get_dst_mac(&self) -> &MacAddress {
        &self.dst_mac
    }

    pub fn get_src_mac(&self) -> &MacAddress {
        &self.src_mac
    }

    pub fn get_type(&self) -> PacketType {
        self.packet_type
    }
}

impl Packet for EthernetPacket {
    fn payload(&self) -> &Vec<u8> {
        &self.payload
    }
}

pub fn handle(packet: &EthernetPacket) {
    println!("Packet Type: {}", packet.get_type());
    match packet.get_type() {
        PacketType::Ipv4 => {
            let ipv4 = packet.payload();
            let ipv4: Ipv4Packet = unsafe { std::ptr::read(ipv4.as_ptr() as *const _) };
            ipv4::handle(&ipv4);
        }
        _ => (),
    }
}
