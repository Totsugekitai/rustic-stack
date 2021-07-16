use std::{fmt, io};

pub const MAC_LENGTH: usize = 6;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress([u8; MAC_LENGTH]);

impl Default for MacAddress {
    fn default() -> MacAddress {
        MacAddress([0; MAC_LENGTH])
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:X?}:{:X?}:{:X?}:{:X?}:{:X?}:{:X?}",
            self[0], self[1], self[2], self[3], self[4], self[5]
        )
    }
}

#[Derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum FrameType {
    Ipv4 = 0x0800,
    Arp = 0x0806,
    AppleTalk = 0x809b,
    Ieee802 = 0x8100,
    Ipx = 0x8137,
    Ipv6 = 0x86dd,
}

impl fmt::Display for FrameType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FrameType::Ipv4 => "IPv4",
                FrameType::Arp => "ARP",
                FrameType::AppleTalk => "AppleTalk",
                FrameType::Ieee802 => "IEEE802",
                FrameType::Ipx => "IPX",
                FrameType::Ipv6 => "IPv6",
            }
        )
    }
}

#[Derive(Debug, Clone, Copy, Eq)]
#[repr(C)]
pub struct EthernetHeader {
    pub dst_mac: MacAddress,
    pub src_mac: MacAddress,
    pub frame_type: FrameType,
}
