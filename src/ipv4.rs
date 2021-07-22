use std::convert::TryInto;
use std::fmt;
use std::num::ParseIntError;

use crate::net::{NetDevice, NetProtocol, NetProtocolErrorKind, NetProtocolType};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv4Address([u8; IPV4_LENGTH]);

pub const IPV4_LENGTH: usize = 4;

impl fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[3], self.0[2], self.0[1], self.0[0],)
    }
}

impl Ipv4Address {
    pub fn from_str(address_str: &str) -> Result<Self, ParseIntError> {
        let ipv4_address: [u8; IPV4_LENGTH] = address_str
            .split(':')
            .map(|s| u8::from_str_radix(s, 16))
            .collect::<Result<Vec<u8>, ParseIntError>>()?
            .try_into()
            .unwrap_or_else(|v: Vec<u8>| {
                panic!(
                    "Expected a Vec of length {}, but it was {}",
                    IPV4_LENGTH,
                    v.len()
                )
            });
        Ok(Ipv4Address(ipv4_address))
    }
}

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

impl Ipv4Packet {
    pub fn version(&self) -> u8 {
        self.version_and_ihl & 0b1111
    }

    pub fn header_length(&self) -> u8 {
        (self.version_and_ihl >> 4) & 0b1111
    }

    pub fn dscp(&self) -> u8 {
        self.dscp_and_ecn & 0b111111
    }

    pub fn ecn(&self) -> u8 {
        (self.dscp_and_ecn >> 6) & 0b11
    }

    pub fn packet_length(&self) -> u16 {
        self.total_length
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn flags(&self) -> u16 {
        self.flags_and_flagment_offset & 0b111
    }

    pub fn fragment_offset(&self) -> u16 {
        self.flags_and_flagment_offset >> 3
    }

    pub fn time_to_live(&self) -> u8 {
        self.time_to_live
    }

    pub fn protocol(&self) -> Protocol {
        match self.protocol {
            1 => Protocol::Icmp,
            4 => Protocol::Ip,
            6 => Protocol::Tcp,
            17 => Protocol::Udp,
            _ => Protocol::Unimplement,
        }
    }

    pub fn src_address(&self) -> Ipv4Address {
        let ipv4_address_0 = ((self.src_ip_address >> 0) & 0xff) as u8;
        let ipv4_address_1 = ((self.src_ip_address >> 8) & 0xff) as u8;
        let ipv4_address_2 = ((self.src_ip_address >> 16) & 0xff) as u8;
        let ipv4_address_3 = ((self.src_ip_address >> 24) & 0xff) as u8;
        Ipv4Address([
            ipv4_address_0,
            ipv4_address_1,
            ipv4_address_2,
            ipv4_address_3,
        ])
    }

    pub fn dst_address(&self) -> Ipv4Address {
        let ipv4_address_0 = ((self.dst_ip_address >> 0) & 0xff) as u8;
        let ipv4_address_1 = ((self.dst_ip_address >> 8) & 0xff) as u8;
        let ipv4_address_2 = ((self.dst_ip_address >> 16) & 0xff) as u8;
        let ipv4_address_3 = ((self.dst_ip_address >> 24) & 0xff) as u8;
        Ipv4Address([
            ipv4_address_0,
            ipv4_address_1,
            ipv4_address_2,
            ipv4_address_3,
        ])
    }
}

#[repr(u8)]
pub enum Protocol {
    Icmp = 1,
    Ip = 4,
    Tcp = 6,
    Udp = 17,
    Unimplement,
}

pub fn handle(_packet: &Ipv4Packet) {}

pub fn input(data: *const u8, size: usize, dev: &mut NetDevice) -> *mut u8 {
    eprintln!("IP input DEV={} SIZE={} ", dev.name, size);
    data as *mut u8
}

pub fn init() {
    let r = NetProtocol::register(NetProtocolType::Ip as u16, input);
    match r {
        Ok(()) => (),
        Err(e) => match e.kind {
            NetProtocolErrorKind::AlreadyRegistered => (),
        },
    }
}
