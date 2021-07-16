use std::fmt;

pub const MAC_LENGTH: usize = 6;
pub const MAC_BROADCAST: [u8; MAC_LENGTH] = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress([u8; MAC_LENGTH]);

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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

#[derive(Debug, Clone)]
#[repr(C)]
pub struct EthernetPacket {
    pub dst_mac: MacAddress,
    pub src_mac: MacAddress,
    pub frame_type: FrameType,
    pub data: Vec<u8>,
}
