use std::collections::VecDeque;
use std::fmt;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, MutexGuard,
};
use std::thread;
use std::time::Duration;

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

pub type ProtocolHandlerType = fn(&Vec<u8>, &'static NetDevice);

pub struct NetProtocol {
    protocol_type: u16,
    queue: Mutex<VecDeque<NetProtocolQueueEntry>>,
    handler: ProtocolHandlerType,
}

impl NetProtocol {
    pub fn register(
        protocol_type: u16,
        handler: ProtocolHandlerType,
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
                queue: Mutex::new(VecDeque::new()),
                handler,
            };
            protocols.push(protocol);
        }
        Ok(())
    }

    pub fn input_handler(
        protocol_type: u16,
        data: *const u8,
        size: usize,
        dev: &'static NetDevice,
    ) -> Result<(), NetProtocolError> {
        {
            let mut protocols = PROTOCOLS.lock();
            for protocol in protocols.iter_mut() {
                if protocol.protocol_type == protocol_type {
                    {
                        let mut queue = protocol.queue.lock().unwrap();
                        unsafe {
                            let data = data as *mut u8;
                            queue.push_back(NetProtocolQueueEntry {
                                dev: dev,
                                data: Vec::from_raw_parts(data, size, size),
                            })
                        };
                    }
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
    dev: &'static NetDevice,
    data: Vec<u8>,
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

pub enum NetDeviceAddress {
    Peer([u8; HARDWARE_ADDRESS_LENGTH]),
    Broadcast([u8; HARDWARE_ADDRESS_LENGTH]),
}

pub type OpenFnPtr = fn(&NetDevice) -> isize;
pub type CloseFnPtr = fn(&NetDevice) -> isize;
pub type TransmitFnPtr = fn(&NetDevice, u16, *const u8, usize, *mut u8) -> isize;
pub type PollFnPtr = fn(&NetDevice) -> isize;

pub struct NetDeviceOps {
    pub open: Option<OpenFnPtr>,
    pub close: Option<CloseFnPtr>,
    pub transmit: Option<TransmitFnPtr>,
    pub poll: Option<PollFnPtr>,
}

impl NetDeviceOps {
    pub fn empty(_: &NetDevice) -> isize {
        0
    }
}

impl NetDevice {
    fn is_up(&self) -> bool {
        self.flags & NetDeviceFlag::Up as u16 > 0
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
        if let Some(open) = self.ops.open {
            if open(&self) == -1 {
                eprintln!("open error DEV={}", self.name);
                return Err(NetDeviceError::new(NetDeviceErrorKind::OpenError));
            }
        }

        self.flags = self.flags | NetDeviceFlag::Up as u16;
        println!("open device DEV={}", self.name);
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), NetDeviceError> {
        if !self.is_up() {
            return Err(NetDeviceError::new(NetDeviceErrorKind::AlreadyDown));
        }
        if let Some(close) = self.ops.close {
            if close(&self) == -1 {
                eprintln!("close error DEV={}", self.name);
                return Err(NetDeviceError::new(NetDeviceErrorKind::CloseError));
            }
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

        if let Some(transmit) = self.ops.transmit {
            if transmit(self, net_device_type, data, size, dst) == -1 {
                eprintln!("data transmit failed DEV={} SIZE={}", self.name, size);
                return Err(NetDeviceError::new(NetDeviceErrorKind::TransmitError));
            }
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

pub struct LockableThreadHandle {
    pub item: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl LockableThreadHandle {
    pub fn new() -> Self {
        LockableThreadHandle {
            item: Arc::new(Mutex::new(Option::from(thread::spawn(|| {})))),
        }
    }

    pub fn lock(&self) -> LockedThreadHandle<'_> {
        LockedThreadHandle {
            item: self.item.lock().unwrap(),
        }
    }
}

pub struct LockedThreadHandle<'a> {
    pub item: MutexGuard<'a, Option<thread::JoinHandle<()>>>,
}

impl<'a> LockedThreadHandle<'a> {
    fn join(&mut self) -> thread::Result<()> {
        self.item.take().unwrap().join()
    }
}

lazy_static! {
    pub static ref THREAD: LockableThreadHandle = LockableThreadHandle::new();
}

static TERMINATE: AtomicBool = AtomicBool::new(false);

pub fn net_thread() {
    while !TERMINATE.load(Ordering::Acquire) {
        let mut count = 0;
        {
            let mut devices = NET_DEVICES.lock();
            for dev in devices.iter_mut() {
                if dev.is_up() {
                    if let Some(poll) = dev.ops.poll {
                        if poll(dev) != -1 {
                            count += 1;
                        }
                    }
                }
            }
        }
        {
            let mut protocols = PROTOCOLS.lock();
            for protocol in protocols.iter_mut() {
                {
                    let mut queue = protocol.queue.lock().unwrap();
                    let entry_option = queue.pop_front();
                    if let Some(entry) = entry_option {
                        let _ = (protocol.handler)(&entry.data, entry.dev);
                        count += 1;
                    }
                }
            }
        }
        if count == 0 {
            thread::sleep(Duration::new(0, 1000_0000));
        }
    }
}

pub fn net_run() -> Result<(), NetDeviceError> {
    let mut net_devices = NET_DEVICES.lock();
    for dev in net_devices.iter_mut() {
        dev.open()?;
    }

    let handle = thread::spawn(|| {
        net_thread();
    });

    {
        let mut thread_handle = THREAD.lock();
        *(thread_handle.item) = Option::from(handle);
    }

    Ok(())
}

pub fn net_shutdown() -> Result<(), NetDeviceError> {
    let mut net_devices = NET_DEVICES.lock();
    for dev in net_devices.iter_mut() {
        dev.close()?;
    }

    let _ = TERMINATE.compare_exchange(false, true, Ordering::Release, Ordering::Relaxed);

    {
        let mut handle = THREAD.lock();
        handle.join().unwrap();
    }

    Ok(())
}

pub fn net_init() {
    ipv4::init();
}
