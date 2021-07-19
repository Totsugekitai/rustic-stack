pub mod linux {
    use crate::ethernet::MacAddress;
    use ifstructs::ifreq;
    use libc;
    // use once_cell::sync::OnceCell;
    use std::fs::{read_dir, read_link, read_to_string, File};
    use std::io;
    use std::os::unix::io::{AsRawFd, RawFd};
    use std::path::Path;

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

    // struct Epoll {
    //     fd: RawFd,
    //     events: Vec<libc::epoll_event>,
    // }

    // enum EpollEventType {
    //     In,
    //     Out,
    // }

    // impl Epoll {
    //     fn new(max_event: usize) -> Self {
    //         let fd = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
    //         let event: libc::epoll_event = unsafe { std::mem::zeroed() };
    //         let events = vec![event; max_event];
    //         Epoll { fd, events }
    //     }

    //     fn add_event(&self, fd: RawFd, op: EpollEventType) {
    //         let mut event: libc::epoll_event = unsafe { std::mem::zeroed() };
    //         event.u64 = fd as u64;
    //         event.events = match op {
    //             EpollEventType::In => libc::EPOLLIN as u32,
    //             EpollEventType::Out => libc::EPOLLOUT as u32,
    //         };

    //         unsafe { libc::epoll_ctl(self.fd, libc::EPOLL_CTL_ADD, fd, &mut event as *mut _) };
    //     }

    //     fn mod_event(&self, fd: RawFd, op: EpollEventType) {
    //         let mut event: libc::epoll_event = unsafe { std::mem::zeroed() };
    //         event.u64 = fd as u64;
    //         event.events = match op {
    //             EpollEventType::In => libc::EPOLLIN as u32,
    //             EpollEventType::Out => libc::EPOLLOUT as u32,
    //         };

    //         unsafe { libc::epoll_ctl(self.fd, libc::EPOLL_CTL_MOD, fd, &mut event as *mut _) };
    //     }

    //     fn del_event(&self, fd: RawFd) {
    //         unsafe {
    //             libc::epoll_ctl(
    //                 self.fd,
    //                 libc::EPOLL_CTL_DEL,
    //                 fd,
    //                 std::ptr::null_mut() as *mut _,
    //             )
    //         };
    //     }

    //     fn wait(&mut self) -> usize {
    //         let nfd = unsafe {
    //             libc::epoll_wait(
    //                 self.fd,
    //                 self.events.as_mut_ptr(),
    //                 self.events.len() as i32,
    //                 -1, // no timeout
    //             )
    //         };

    //         nfd as usize
    //     }
    // }

    #[derive(Debug)]
    pub struct Tap {
        fd: RawFd,
        name: String,
        ip_address: [u8; 4],
        mac_address: MacAddress,
    }

    // static TAP: OnceCell<Tap> = OnceCell::new();

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
                ip_address: [192, 0, 2, 1],
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
