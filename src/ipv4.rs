use std::convert::TryInto;
use std::fmt;
use std::num::ParseIntError;
use std::{io, io::Write};

use crate::net::{NetDevice, NetProtocol, NetProtocolErrorKind, NetProtocolType};

pub const IP_HEADER_SIZE_MIN: u16 = 20;
pub const IP_HEADER_SIZE_MAX: u16 = 60;

pub const IP_VERSION_IPV4: u8 = 4;

pub const IP_TOTAL_SIZE_MAX: u16 = u16::MAX;
pub const IP_PAYLOAD_SIZE_MAX: u16 = IP_TOTAL_SIZE_MAX - IP_HEADER_SIZE_MIN;

pub const IPV4_ADDRESS_SIZE: usize = 4;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv4Address([u8; IPV4_ADDRESS_SIZE]);

impl fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[3], self.0[2], self.0[1], self.0[0],)
    }
}

impl Ipv4Address {
    pub fn from_str(address_str: &str) -> Result<Self, ParseIntError> {
        let ipv4_address: [u8; IPV4_ADDRESS_SIZE] = address_str
            .split(':')
            .map(|s| u8::from_str_radix(s, 16))
            .collect::<Result<Vec<u8>, ParseIntError>>()?
            .try_into()
            .unwrap_or_else(|v: Vec<u8>| {
                panic!(
                    "Expected a Vec of length {}, but it was {}",
                    IPV4_ADDRESS_SIZE,
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
        ((self.vhl >> 4) & 0b1111) << 2
    }

    pub fn type_of_service(&self) -> u8 {
        self.tos
    }

    pub fn dscp(&self) -> u8 {
        self.tos & 0b111111
    }

    pub fn ecn(&self) -> u8 {
        (self.tos >> 6) & 0b11
    }

    pub fn total_length(&self) -> u16 {
        u16::from_be(self.total_length)
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn flags(&self) -> u16 {
        let offset = u16::from_be(self.offset);
        offset >> 13
    }

    pub fn offset(&self) -> u16 {
        let offset = u16::from_be(self.offset);
        offset & 0b0001_1111_1111_1111
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

    pub fn checksum(&self) -> u16 {
        u16::from_be(self.sum)
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

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Protocol::Icmp => {
                write!(f, "ICMP")
            }
            Protocol::Ip => {
                write!(f, "IP")
            }
            Protocol::Tcp => {
                write!(f, "TCP")
            }
            Protocol::Udp => {
                write!(f, "UDP")
            }
            Protocol::Unimplement => {
                write!(f, "Unimplement")
            }
        }
    }
}

pub fn dump(data: *const u8, size: usize) -> io::Result<()> {
    let stderr = io::stderr();
    let mut handle = stderr.lock();

    let ipv4_hdr = unsafe { data.cast::<Ipv4Header>().as_ref().unwrap() };

    write!(handle, "IPv4 Header ==========")?;
    write!(
        handle,
        "            vhl: 0x{:02x} [version: {}, header length: {}]",
        ipv4_hdr.vhl,
        ipv4_hdr.version(),
        ipv4_hdr.header_length()
    )?;
    write!(
        handle,
        "type of service: 0x{:02x}",
        ipv4_hdr.type_of_service(),
    )?;

    write!(
        handle,
        "   total length: 0x{:x} (payload 0x{:x})",
        ipv4_hdr.total_length(),
        ipv4_hdr.total_length() - ipv4_hdr.header_length() as u16
    )?;
    write!(handle, "             id: {:x}", ipv4_hdr.id(),)?;

    write!(handle, "           flag: 0x{:x}", ipv4_hdr.flags())?;
    write!(handle, "         offset: 0x{:x}", ipv4_hdr.offset())?;
    write!(handle, "   time to live: 0x{:x}", ipv4_hdr.time_to_live())?;
    write!(handle, "       protocol: {}", ipv4_hdr.protocol())?;
    write!(handle, "       checksum: 0x{:04x}", ipv4_hdr.checksum())?;
    write!(handle, "    src address: {}", ipv4_hdr.src_address())?;
    write!(handle, "    dst address: {}", ipv4_hdr.src_address())?;

    handle.flush()
}

pub fn handle(_packet: &Ipv4Header) {}

pub fn input(data: &Vec<u8>, dev: &'static NetDevice) {
    if data.len() < IP_HEADER_SIZE_MIN as usize {
        eprintln!("IP header too short");
        return;
    }

    let ipv4_hdr = unsafe { &*(data.as_ptr() as *const Ipv4Header) };

    if ipv4_hdr.version() != IP_VERSION_IPV4 {
        eprintln!("IP version error: {}", ipv4_hdr.version());
        return;
    }
    if data.len() < ipv4_hdr.header_length() as usize {
        eprintln!(
            "IP header length error: header length={}, length={}",
            ipv4_hdr.header_length(),
            data.len()
        );
        return;
    }
    if data.len() < ipv4_hdr.total_length() as usize {
        eprintln!(
            "IP packet total length error: total={}, length={}",
            ipv4_hdr.total_length(),
            data.len()
        );
        return;
    }
    if ipv4_hdr.time_to_live() == 0 {
        eprintln!("Time exceeded (TTL=0)");
        return;
    }

    eprintln!(
        "IP input DEV={} PROTOCOL={} TOTAL={} ",
        dev.name,
        ipv4_hdr.protocol(),
        ipv4_hdr.total_length()
    );
    let _ = dump(data.as_ptr(), data.len());
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
