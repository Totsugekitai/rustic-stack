pub mod linux {
    use crate::ethernet::MacAddress;
    use crate::ipv4::Ipv4Address;
    use ifstructs::ifreq;
    use libc;
    use std::fs::{read_dir, read_link, read_to_string, File};
    use std::io;
    use std::os::unix::io::{AsRawFd, RawFd};
    use std::path::Path;
    use std::sync::{Arc, Mutex};

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
        pub static ref NET_DEVICES: Arc<Vec<Mutex<NetDevice>>> = Arc::new(Vec::new());
    }

    pub const HARDWARE_ADDRESS_LENGTH: usize = 16;

    #[repr(C)]
    pub struct NetDevice {
        name: String,
        device_type: NetDeviceType,
        mtu: u16,
        flags: u16,
        header_length: u16,
        address_length: u16,
        hwaddr: [u8; HARDWARE_ADDRESS_LENGTH],
        pb: NetDeviceAddress,
        ops: NetDeviceOps,
    }

    #[repr(u16)]
    pub enum NetDeviceType {
        Null = 0x0000,
        LoopBack = 0x0001,
        Ethernet = 0x0002,
    }

    pub enum NetDeviceAddress {
        Peer([u8; HARDWARE_ADDRESS_LENGTH]),
        Broadcast([u8; HARDWARE_ADDRESS_LENGTH]),
    }

    pub struct NetDeviceOps {
        open: fn(&NetDevice) -> isize,
        close: fn(&NetDevice) -> isize,
        transmit: fn(&NetDevice, u16, *const u8, usize, *const u8) -> isize,
        poll: fn(&NetDevice) -> isize,
    }

    impl NetDevice {
        pub fn register(dev: NetDevice) {}
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

    // static TAP: OnceCell<Tap> = OnceCell::new();
}
