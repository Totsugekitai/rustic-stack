pub mod linux {
    use libc;
    use once_cell::sync::OnceCell;
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

    struct Epoll {
        fd: RawFd,
        events: Vec<libc::epoll_event>,
    }

    enum EpollEventType {
        In,
        Out,
    }

    impl Epoll {
        fn new(max_event: usize) -> Self {
            let fd = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
            let event: libc::epoll_event = unsafe { std::mem::zeroed() };
            let events = vec![event; max_event];
            Epoll { fd, events }
        }

        fn add_event(&self, fd: RawFd, op: EpollEventType) {
            let mut event: libc::epoll_event = unsafe { std::mem::zeroed() };
            event.u64 = fd as u64;
            event.events = match op {
                EpollEventType::In => libc::EPOLLIN as u32,
                EpollEventType::Out => libc::EPOLLOUT as u32,
            };

            unsafe { libc::epoll_ctl(self.fd, libc::EPOLL_CTL_ADD, fd, &mut event as *mut _) };
        }

        fn mod_event(&self, fd: RawFd, op: EpollEventType) {
            let mut event: libc::epoll_event = unsafe { std::mem::zeroed() };
            event.u64 = fd as u64;
            event.events = match op {
                EpollEventType::In => libc::EPOLLIN as u32,
                EpollEventType::Out => libc::EPOLLOUT as u32,
            };

            unsafe { libc::epoll_ctl(self.fd, libc::EPOLL_CTL_MOD, fd, &mut event as *mut _) };
        }

        fn del_event(&self, fd: RawFd) {
            unsafe {
                libc::epoll_ctl(
                    self.fd,
                    libc::EPOLL_CTL_DEL,
                    fd,
                    std::ptr::null_mut() as *mut _,
                )
            };
        }

        fn wait(&mut self) -> usize {
            let nfd = unsafe {
                libc::epoll_wait(
                    self.fd,
                    self.events.as_mut_ptr(),
                    self.events.len() as i32,
                    -1, // no timeout
                )
            };

            nfd as usize
        }
    }

    const IFNAMSIZ: usize = 16;

    #[repr(C)]
    struct Ifreq {
        ifr_name: [u8; IFNAMSIZ],
        ifr_ifru: [u8; IFNAMSIZ],
    }

    static TAP_FILE: OnceCell<File> = OnceCell::new();

    pub fn open_tap() -> io::Result<()> {
        let tap_path = Path::new("/dev/net/tun");
        let tap_file = File::open(tap_path)?;

        TAP_FILE.set(tap_file).unwrap();
        Ok(())
    }

    pub const RW_BUF_SIZE: usize = 2 << 10;

    pub fn poll_tap(v: &mut [u8; RW_BUF_SIZE], size: usize) {
        let mut epoll = Epoll::new(2); // ここの引数はテキトー

        loop {
            let nfd = epoll.wait();

            for i in 0..nfd {
                let fd = epoll.events[i].u64 as RawFd;
                let events = epoll.events[i].events as i32;

                if (events & libc::EPOLLIN) > 0 {
                    let n = read_tap(v, size);
                    if n == 0 {
                        epoll.del_event(fd);
                        break;
                    }
                } else if (events & libc::EPOLLOUT) > 0 {
                    let _n = write_tap(v, size);
                }
            }
        }
    }

    pub fn read_tap(buf: &mut [u8; RW_BUF_SIZE], size: usize) -> isize {
        unsafe {
            libc::read(
                TAP_FILE.get().unwrap().as_raw_fd(),
                buf as *mut _ as *mut libc::c_void,
                size as _,
            ) as isize
        }
    }

    pub fn write_tap(data: &mut [u8; RW_BUF_SIZE], size: usize) -> isize {
        unsafe {
            libc::write(
                TAP_FILE.get().unwrap().as_raw_fd(),
                data as *mut _ as *mut libc::c_void,
                size as _,
            ) as isize
        }
    }

    pub fn close_tap() {
        unsafe { libc::close(TAP_FILE.get().unwrap().as_raw_fd()) };
    }
}
