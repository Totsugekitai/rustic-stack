use std::fs::File;
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::Path;

use ifstructs::ifreq;

use crate::ethernet::MacAddress;
use crate::ipv4::Ipv4Address;

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
