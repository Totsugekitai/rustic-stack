use std::convert::TryInto;
use std::fmt;
use std::num::ParseIntError;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{io, io::Write};

use crate::net::{
    NetDevice, NetDeviceErrorKind, NetInterface, NetInterfaceFamily, NetInterfaceType, NetProtocol,
    NetProtocolErrorKind, NetProtocolType,
};
use crate::utils::checksum16;

pub const IP_HEADER_SIZE_MIN: u16 = 20;
pub const IP_HEADER_SIZE_MAX: u16 = 60;

pub const IP_VERSION_IPV4: u8 = 4;

pub const IP_TOTAL_SIZE_MAX: u16 = u16::MAX;
pub const IP_PAYLOAD_SIZE_MAX: u16 = IP_TOTAL_SIZE_MAX - IP_HEADER_SIZE_MIN;

pub const IPV4_ADDRESS_SIZE: usize = 4;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ipv4Address([u8; IPV4_ADDRESS_SIZE]);

pub const IP_ADDRESS_BROADCAST: Ipv4Address = Ipv4Address([255; IPV4_ADDRESS_SIZE]);

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

    pub fn to_u32(&self) -> u32 {
        ((self.0[3] as u32) << 24)
            | ((self.0[2] as u32) << 16)
            | ((self.0[1] as u32) << 8)
            | ((self.0[0] as u32) << 0)
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

pub struct LockableIpInterfaces {
    pub items: Arc<Mutex<Vec<Option<&'static IpInterface>>>>,
}

impl LockableIpInterfaces {
    pub fn new() -> Self {
        LockableIpInterfaces {
            items: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn lock(&self) -> LockedIpInterfaces<'_> {
        LockedIpInterfaces {
            items: self.items.lock().unwrap(),
        }
    }
}

pub struct LockedIpInterfaces<'a> {
    pub items: MutexGuard<'a, Vec<Option<&'static IpInterface>>>,
}

impl<'a> LockedIpInterfaces<'a> {
    fn iter(&self) -> impl Iterator<Item = &Option<&'static IpInterface>> {
        self.items.iter()
    }

    // fn iter_mut(&mut self) -> impl Iterator<Item = &mut Option<&'static IpInterface>> {
    //     self.items.iter_mut()
    // }
}

lazy_static! {
    pub static ref IP_INTERFACES: LockableIpInterfaces = LockableIpInterfaces::new();
}

pub struct IpInterface {
    pub net_interface: NetInterface,
    pub unicast: Ipv4Address,
    pub netmask: Ipv4Address,
    pub broadcast: Ipv4Address,
}

impl IpInterface {
    pub fn alloc(unicast: &str, netmask: &str) -> Option<Box<Self>> {
        let mut interface = Box::new(IpInterface::default());
        interface.net_interface.family = NetInterfaceFamily::Ip;
        if let Ok(addr) = Ipv4Address::from_str(unicast) {
            interface.unicast = addr;
        } else {
            eprintln!("Invalid unicast IP address");
            return None;
        }

        if let Ok(addr) = Ipv4Address::from_str(netmask) {
            interface.netmask = addr;
        } else {
            eprintln!("Invalid netmask");
            return None;
        }

        interface.broadcast = Ipv4Address::from_u32(
            (interface.unicast.to_u32() & interface.netmask.to_u32()) | !interface.netmask.to_u32(),
        );

        Some(interface)
    }

    pub fn register(ip_interface: Option<&'static Self>, dev: &mut NetDevice) -> Result<(), ()> {
        {
            if let None = ip_interface {
                return Err(());
            }
            let mut interfaces = IP_INTERFACES.lock();
            interfaces.items.push(ip_interface);
            if let Err(e) = dev.add_interface(NetInterfaceType::Ip(ip_interface.unwrap())) {
                match e.kind {
                    NetDeviceErrorKind::AlreadyRegistered => {
                        return Ok(());
                    }
                    _ => {
                        eprintln!("add interface is failed");
                        return Err(());
                    }
                }
            }
        }

        Ok(())
    }

    pub fn select(address: Ipv4Address) -> Option<&'static IpInterface> {
        let interfaces = IP_INTERFACES.lock();
        for entry in interfaces.iter() {
            let entry = entry.unwrap();
            if entry.unicast == address {
                return Some(entry);
            }
        }
        None
    }
}

impl Default for IpInterface {
    fn default() -> Self {
        Self {
            net_interface: NetInterface::default(),
            unicast: Ipv4Address::from_str("0.0.0.0").unwrap(),
            netmask: Ipv4Address::from_str("0.0.0.0").unwrap(),
            broadcast: Ipv4Address::from_str("0.0.0.0").unwrap(),
        }
    }
}

pub fn dump(data: *const u8, _size: usize) -> io::Result<()> {
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

    if checksum16(
        ipv4_hdr as *const Ipv4Header as *const u16,
        ipv4_hdr.header_length() as u16,
        0,
    ) != 0
    {
        eprintln!("Checksum error");
        return;
    }

    let interface = dev.get_interface(NetInterfaceFamily::Ip);
    if let Some(interface) = interface {
        match interface {
            NetInterfaceType::Ip(ip_interface) => {
                if ip_interface.unicast != ipv4_hdr.dst_address() {
                    return;
                }
                if (ipv4_hdr.dst_address() != ip_interface.broadcast)
                    && ipv4_hdr.dst_address() != IP_ADDRESS_BROADCAST
                {}
            }
            NetInterfaceType::Unknown => {
                return;
            }
        }
    }

    let offset = ipv4_hdr.offset();
    if (offset & 0x2000 > 0) || (offset & 0x1fff > 0) {
        eprintln!("fragment is not supported");
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
