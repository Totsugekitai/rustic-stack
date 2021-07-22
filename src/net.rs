use std::fmt;
use std::ops;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::ipv4;

#[repr(u16)]
pub enum NetProtocolType {
    Ip = 0x0800,
    Arp = 0x0806,
    Ipv6 = 0x86dd,
    Unknown,
}

impl NetProtocolType {
    pub fn from_u16(u: u16) -> NetProtocolType {
        let u = u & 0xfffe;
        match u {
            0x0800 => NetProtocolType::Ip,
            0x0806 => NetProtocolType::Arp,
            0x86dd => NetProtocolType::Ipv6,
            _ => NetProtocolType::Unknown,
        }
    }
}

impl fmt::Display for NetProtocolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NetProtocolType::Ip => "IPv4",
            NetProtocolType::Arp => "ARP",
            NetProtocolType::Ipv6 => "IPv6",
            NetProtocolType::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

pub struct LockableNetProtocols {
    pub items: Arc<Mutex<Vec<NetProtocol>>>,
}

impl LockableNetProtocols {
    pub fn new() -> Self {
        LockableNetProtocols {
            items: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn lock(&self) -> LockedNetProtocols<'_> {
        LockedNetProtocols {
            items: self.items.lock().unwrap(),
        }
    }
}

pub struct LockedNetProtocols<'a> {
    pub items: MutexGuard<'a, Vec<NetProtocol>>,
}

impl<'a> LockedNetProtocols<'a> {
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut NetProtocol> {
        self.items.iter_mut()
    }

    fn push(&mut self, protocol: NetProtocol) {
        self.items.push(protocol);
    }
}

lazy_static! {
    pub static ref PROTOCOLS: LockableNetProtocols = LockableNetProtocols::new();
}

pub struct NetProtocolError {
    pub kind: NetProtocolErrorKind,
}

pub enum NetProtocolErrorKind {
    AlreadyRegistered,
}

// type T is Protocol Type
pub struct NetProtocol {
    protocol_type: u16,
    queue: Vec<NetProtocolQueueEntry>,
    handler: fn(*const u8, usize, &mut NetDevice) -> *mut u8,
}

impl NetProtocol {
    pub fn register(
        protocol_type: u16,
        handler: fn(*const u8, usize, &mut NetDevice) -> *mut u8,
    ) -> Result<(), NetProtocolError> {
        {
            let mut protocols = PROTOCOLS.lock();
            for protocol in protocols.iter_mut() {
                if protocol.protocol_type == protocol_type {
                    eprintln!("protocol is already registered TYPE={:04x}", protocol_type);
                    return Err(NetProtocolError {
                        kind: NetProtocolErrorKind::AlreadyRegistered,
                    });
                }
            }
            let protocol = NetProtocol {
                protocol_type,
                queue: Vec::new(),
                handler,
            };
            protocols.push(protocol);
        }
        Ok(())
    }

    pub fn input_handler(
        protocol_type: u16,
        _data: *const u8,
        size: usize,
        dev: &'static NetDevice,
    ) -> Result<(), NetProtocolError> {
        {
            let mut protocols = PROTOCOLS.lock();
            for protocol in protocols.iter_mut() {
                if protocol.protocol_type == protocol_type {
                    protocol.queue.push(NetProtocolQueueEntry {
                        _dev: dev,
                        _size: size,
                    });
                }
            }
        }
        println!(
            "Queue pushed DEV={} TYPE={}:{:04x} SIZE={}",
            dev.name,
            NetProtocolType::from_u16(protocol_type),
            protocol_type,
            size
        );
        Ok(())
    }
}

pub struct NetProtocolQueueEntry {
    _dev: &'static NetDevice,
    _size: usize,
}

pub const HARDWARE_ADDRESS_LENGTH: usize = 16;

pub struct NetDeviceError {
    _kind: NetDeviceErrorKind,
}

impl NetDeviceError {
    pub fn new(kind: NetDeviceErrorKind) -> Self {
        Self { _kind: kind }
    }
}

pub enum NetDeviceErrorKind {
    AlreadyUp,
    AlreadyDown,
    OpenError,
    CloseError,
    TransmitError,
    DataSizeTooBig,
}

pub struct NetDevice {
    pub name: String,
    pub device_type: u16,
    pub mtu: u16,
    pub flags: u16,
    pub header_length: u16,
    pub address_length: u16,
    pub hwaddr: [u8; HARDWARE_ADDRESS_LENGTH],
    pub pb: NetDeviceAddress,
    pub ops: NetDeviceOps,
}

#[derive(PartialEq, Eq)]
#[repr(u16)]
pub enum NetDeviceType {
    Null = 0x0000,
    Loopback = 0x0001,
    Ethernet = 0x0002,
    Unknown,
}

impl NetDeviceType {
    pub fn from_u16(u: u16) -> NetDeviceType {
        let u = u & 0b11;
        match u {
            0 => NetDeviceType::Null,
            1 => NetDeviceType::Loopback,
            2 => NetDeviceType::Ethernet,
            _ => NetDeviceType::Unknown,
        }
    }
}

impl fmt::Display for NetDeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            NetDeviceType::Null => "Null",
            NetDeviceType::Loopback => "Loopback",
            NetDeviceType::Ethernet => "Ethernet",
            NetDeviceType::Unknown => "Unknown",
        };
        write!(f, "{}", s)
    }
}

#[repr(u16)]
pub enum NetDeviceFlag {
    Up = 0x0001,
    Loopback = 0x0010,
    Broadcast = 0x0020,
    P2P = 0x0040,
    NeedArp = 0x0100,
}

impl ops::BitAnd<u16> for NetDeviceFlag {
    type Output = u16;

    fn bitand(self, word: u16) -> u16 {
        word & (self as u16)
    }
}

impl ops::BitAnd<NetDeviceFlag> for u16 {
    type Output = u16;

    fn bitand(self, flag: NetDeviceFlag) -> u16 {
        flag & (self as u16)
    }
}

impl ops::BitOr<u16> for NetDeviceFlag {
    type Output = u16;

    fn bitor(self, word: u16) -> u16 {
        word | (self as u16)
    }
}

impl ops::BitOr<NetDeviceFlag> for u16 {
    type Output = u16;

    fn bitor(self, flag: NetDeviceFlag) -> u16 {
        flag | (self as u16)
    }
}

pub enum NetDeviceAddress {
    Peer([u8; HARDWARE_ADDRESS_LENGTH]),
    Broadcast([u8; HARDWARE_ADDRESS_LENGTH]),
}

pub struct NetDeviceOps {
    pub open: fn(&NetDevice) -> isize,
    pub close: fn(&NetDevice) -> isize,
    pub transmit: fn(&NetDevice, u16, *const u8, usize, *mut u8) -> isize,
    pub poll: fn(&NetDevice) -> isize,
}

impl NetDeviceOps {
    pub fn empty(_: &NetDevice) -> isize {
        0
    }
}

impl NetDevice {
    fn is_up(&self) -> bool {
        self.flags & NetDeviceFlag::Up > 0
    }

    pub fn register(dev: NetDevice) {
        println!("net device register DEV={}", dev.name);
        let mut net_devices = NET_DEVICES.lock();
        net_devices.items.push(dev);
    }

    pub fn open(&mut self) -> Result<(), NetDeviceError> {
        if self.is_up() {
            eprintln!("device is already up DEV={}", self.name);
            return Err(NetDeviceError::new(NetDeviceErrorKind::AlreadyUp));
        }
        if (self.ops.open)(&self) == -1 {
            eprintln!("open error DEV={}", self.name);
            return Err(NetDeviceError::new(NetDeviceErrorKind::OpenError));
        }

        self.flags = self.flags | NetDeviceFlag::Up;
        println!("open device DEV={}", self.name);
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), NetDeviceError> {
        if !self.is_up() {
            return Err(NetDeviceError::new(NetDeviceErrorKind::AlreadyDown));
        }
        if (self.ops.close)(&self) == -1 {
            eprintln!("close error DEV={}", self.name);
            return Err(NetDeviceError::new(NetDeviceErrorKind::CloseError));
        }

        self.flags = self.flags & !(NetDeviceFlag::Up as u16);
        Ok(())
    }

    pub fn output(
        &self,
        net_device_type: u16,
        data: *const u8,
        size: usize,
        dst: *mut u8,
    ) -> Result<(), NetDeviceError> {
        if !self.is_up() {
            eprintln!("not opened DEV={}", self.name);
            return Err(NetDeviceError::new(NetDeviceErrorKind::OpenError));
        }

        if size > self.mtu as usize {
            eprintln!(
                "data size too big DEV={} MTU={} SIZE={}",
                self.name, self.mtu, size
            );
            return Err(NetDeviceError::new(NetDeviceErrorKind::DataSizeTooBig));
        }

        if (self.ops.transmit)(self, net_device_type, data, size, dst) == -1 {
            eprintln!("data transmit failed DEV={} SIZE={}", self.name, size);
            return Err(NetDeviceError::new(NetDeviceErrorKind::TransmitError));
        }
        Ok(())
    }

    pub fn input_handler(&self, net_device_type: NetDeviceType, _data: *const u8, size: usize) {
        println!(
            "DEV={} TYPE={} DATA_SIZE={}",
            self.name, net_device_type, size
        );
    }
}

pub struct LockableNetDevices {
    pub items: Arc<Mutex<Vec<NetDevice>>>,
}

impl LockableNetDevices {
    pub fn new() -> Self {
        LockableNetDevices {
            items: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn lock(&self) -> LockedNetDevices<'_> {
        LockedNetDevices {
            items: self.items.lock().unwrap(),
        }
    }
}

pub struct LockedNetDevices<'a> {
    pub items: MutexGuard<'a, Vec<NetDevice>>,
}

impl<'a> LockedNetDevices<'a> {
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut NetDevice> {
        self.items.iter_mut()
    }
}

lazy_static! {
    pub static ref NET_DEVICES: LockableNetDevices = LockableNetDevices::new();
}

pub fn net_run() -> Result<(), NetDeviceError> {
    let mut net_devices = NET_DEVICES.lock();
    for dev in net_devices.iter_mut() {
        dev.open()?;
    }
    Ok(())
}

pub fn net_shutdown() -> Result<(), NetDeviceError> {
    let mut net_devices = NET_DEVICES.lock();
    for dev in net_devices.iter_mut() {
        dev.close()?;
    }
    Ok(())
}

pub fn net_init() {
    ipv4::init();
}
