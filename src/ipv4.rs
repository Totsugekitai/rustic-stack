use std::convert::TryInto;
use std::fmt;
use std::num::ParseIntError;
use std::{io, io::Write};

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

    pub fn from_u32(u: u32) -> Self {
        Ipv4Address([
            ((u >> 0) & 0xff) as u8,
            ((u >> 8) & 0xff) as u8,
            ((u >> 16) & 0xff) as u8,
            ((u >> 24) & 0xff) as u8,
        ])
    }
}

#[repr(C, packed)]
pub struct Ipv4Header {
    vhl: u8,
    tos: u8,
    total_length: u16,
    id: u16,
    offset: u16,
    time_to_live: u8,
    protocol: u8,
    sum: u16,
    src_ip_address: Ipv4Address,
    dst_ip_address: Ipv4Address,
}

impl Ipv4Header {
    pub fn version(&self) -> u8 {
        self.vhl & 0b1111
    }

    pub fn header_length(&self) -> u8 {
        (self.vhl >> 4) & 0b1111
    }

    pub fn dscp(&self) -> u8 {
        self.tos & 0b111111
    }

    pub fn ecn(&self) -> u8 {
        (self.tos >> 6) & 0b11
    }

    pub fn packet_length(&self) -> u16 {
        self.total_length
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn flags(&self) -> u16 {
        self.offset & 0b111
    }

    pub fn fragment_offset(&self) -> u16 {
        self.offset >> 3
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
        self.src_ip_address
    }

    pub fn dst_address(&self) -> Ipv4Address {
        self.dst_ip_address
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

pub fn dump(data: *const u8, size: usize) -> io::Result<()> {
    let stderr = io::stderr();
    let mut handle = stderr.lock();

    let ipv4_hdr = unsafe { data.cast::<Ipv4Header>().as_ref().unwrap() };
    let hlen = ipv4_hdr.header_length() << 2;

    write!(handle, "IPv4 Header ==========")?;
    write!(
        handle,
        "        vhl: 0x{:02x} [v: {}]",
        ipv4_hdr.vhl,
        ipv4_hdr.version(),
    )?;
    let _ = handle.flush();
    Ok(())
}

pub fn handle(_packet: &Ipv4Header) {}

pub fn input(data: &Vec<u8>, dev: &'static NetDevice) {
    eprintln!("IP input DEV={} SIZE={} ", dev.name, data.len());
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
