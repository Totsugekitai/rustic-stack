pub mod linux {
    use crate::ethernet::MacAddress;
    use crate::ipv4::Ipv4Address;
    use ifstructs::ifreq;
    use libc;
    use std::collections::HashMap;
    use std::fmt;
    use std::fs::{read_dir, read_link, read_to_string, File};
    use std::io;
    use std::ops;
    use std::os::unix::io::{AsRawFd, RawFd};
    use std::path::Path;
    use std::sync::Mutex;

    fn read_mac_address(address_file: &Path) -> Result<String, io::Error> {
        let mac_address = read_to_string(address_file)?;
        Ok(mac_address)
    }

    pub fn get_mac_address_list() -> Result<Vec<String>, io::Error> {
        let net_devices_root_dir = Path::new("/sys/class/net");
        let entries = read_dir(net_devices_root_dir)?;
        let mut address_vec = Vec::new();
        for entry in entries {
            let entry = entry?;

            if let Ok(symlink) = read_link(entry.path()) {
                let path = symlink.as_path();
                let address_file_path = path.join("address");
                let mac_address = read_mac_address(&address_file_path)?;
                address_vec.push(mac_address);
            }
        }
        Ok(address_vec)
    }

    lazy_static! {
        pub static ref NET_DEVICES: Mutex<HashMap<String, NetDevice>> = Mutex::new(HashMap::new());
    }

    pub const HARDWARE_ADDRESS_LENGTH: usize = 16;

    pub struct NetDeviceError {
        kind: NetDeviceErrorKind,
    }

    impl NetDeviceError {
        pub fn new(kind: NetDeviceErrorKind) -> Self {
            Self { kind }
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

    #[repr(C)]
    pub struct NetDevice {
        pub name: String,
        pub device_type: NetDeviceType,
        pub mtu: u16,
        pub flags: u16,
        pub header_length: u16,
        pub address_length: u16,
        pub hwaddr: [u8; HARDWARE_ADDRESS_LENGTH],
        pub pb: NetDeviceAddress,
        pub ops: NetDeviceOps,
    }

    #[repr(u16)]
    pub enum NetDeviceType {
        Null = 0x0000,
        Loopback = 0x0001,
        Ethernet = 0x0002,
    }

    impl fmt::Display for NetDeviceType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let s = match self {
                NetDeviceType::Null => "Null",
                NetDeviceType::Loopback => "Loopback",
                NetDeviceType::Ethernet => "Ethernet",
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
        pub transmit: fn(&NetDevice, u16, *const u8, usize, *const u8) -> isize,
        pub poll: fn(&NetDevice) -> isize,
    }

    impl NetDevice {
        fn is_up(&self) -> bool {
            self.flags & NetDeviceFlag::Up > 0
        }

        pub fn register(dev: NetDevice) {
            let mut net_devices = NET_DEVICES.lock().unwrap();
            net_devices.insert(String::from(&dev.name), dev);
        }

        pub fn open(&mut self) -> Result<(), NetDeviceError> {
            if self.is_up() {
                return Err(NetDeviceError::new(NetDeviceErrorKind::AlreadyUp));
            }
            if (self.ops.open)(&self) == -1 {
                eprintln!("[error] device={}", self.name);
                return Err(NetDeviceError::new(NetDeviceErrorKind::OpenError));
            }

            self.flags = self.flags | NetDeviceFlag::Up;
            Ok(())
        }

        pub fn close(&mut self) -> Result<(), NetDeviceError> {
            if !self.is_up() {
                return Err(NetDeviceError::new(NetDeviceErrorKind::AlreadyDown));
            }
            if (self.ops.close)(&self) == -1 {
                eprintln!("[error] device={}", self.name);
                return Err(NetDeviceError::new(NetDeviceErrorKind::CloseError));
            }

            self.flags = self.flags & !(NetDeviceFlag::Up as u16);
            Ok(())
        }

        pub fn output(
            &self,
            net_device_type: NetDeviceType,
            data: *const u8,
            size: usize,
            dst: *const u8,
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

            if (self.ops.transmit)(self, net_device_type as u16, data, size, dst) == -1 {
                eprintln!("data transmit failed DEV={} SIZE={}", self.name, size);
                return Err(NetDeviceError::new(NetDeviceErrorKind::TransmitError));
            }
            Ok(())
        }

        pub fn input_handler(&self, net_device_type: NetDeviceType, data: *const u8, size: usize) {
            println!(
                "DEV={} TYPE={} DATA_SIZE={}",
                self.name, net_device_type, size
            );
        }

        pub fn run() -> Result<(), NetDeviceError> {
            let mut net_devices = NET_DEVICES.lock().unwrap().borrow_mut();
            for (_name, dev) in net_devices {}
        }
    }

    #[derive(Debug)]
    pub struct Tap {
        fd: RawFd,
        name: String,
        ip_address: Ipv4Address,
        mac_address: MacAddress,
    }

    impl Tap {
        pub fn open(tap_name: &str) -> io::Result<Tap> {
            let tap_file = File::open(Path::new("/dev/net/tun"))?;

            let mut ifr = ifreq::from_name(tap_name)?;
            ifr.set_flags((libc::IFF_TAP | libc::IFF_NO_PI) as libc::c_short);

            let tap_fd = tap_file.as_raw_fd();
            let err = unsafe { libc::ioctl(tap_fd, 202, &ifr as *const ifreq) };
            if err < 0 {
                panic!("TAP allocation is failed");
            }

            let tap = Tap {
                fd: tap_fd,
                name: String::from(tap_name),
                ip_address: Ipv4Address::from_str("192.0.2.2").unwrap(),
                mac_address: MacAddress::from_str("00:00:5e:00:53:FF").unwrap(),
            };

            Ok(tap)
        }

        pub fn close(&self) {
            unsafe { libc::close(self.fd) };
        }

        fn read(&self, buf: *mut u8, size: usize) -> isize {
            unsafe { libc::read(self.fd, buf as *mut libc::c_void, size as _) as isize }
        }

        fn write(&self, data: *const u8, size: usize) -> isize {
            unsafe { libc::write(self.fd, data as *const libc::c_void, size as _) as isize }
        }

        pub fn poll(&self, buf: *mut u8, size: usize) -> isize {
            let mut pfd = libc::pollfd {
                fd: self.fd,
                events: libc::POLLIN,
                revents: 0,
            };

            let ret = unsafe { libc::poll(&mut pfd as *mut libc::pollfd, 1, 3000) }; // 3sec待つ
            if ret < 1 {
                eprintln!("poll failed");
            }

            self.read(buf, size)
        }

        pub fn transmit(&self, data: *const u8, size: usize) -> isize {
            self.write(data, size)
        }
    }
}
