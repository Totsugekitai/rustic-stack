use std::ptr;
use std::u16;

use crate::net::{
    NetDevice, NetDeviceAddress, NetDeviceFlag, NetDeviceOps, NetDeviceType,
    HARDWARE_ADDRESS_LENGTH,
};

const LOOPBACK_MTU: u16 = u16::MAX;

pub struct Loopback();

impl Loopback {
    pub fn transmit(
        dev: &NetDevice,
        net_device_type: u16,
        data: *const u8,
        size: usize,
        dst: *mut u8,
    ) -> isize {
        eprintln!(
            "DEV={} TYPE={} SIZE={}",
            dev.name,
            NetDeviceType::from_u16(net_device_type),
            size
        );
        unsafe { ptr::copy(data, dst, size) };
        size as isize
    }

    pub fn new_device() -> NetDevice {
        NetDevice {
            name: String::from("loopback"),
            device_type: NetDeviceType::Loopback,
            mtu: LOOPBACK_MTU,
            flags: NetDeviceFlag::Loopback as u16,
            header_length: 0,
            address_length: 0,
            hwaddr: [0; HARDWARE_ADDRESS_LENGTH],
            pb: NetDeviceAddress::Peer([0; HARDWARE_ADDRESS_LENGTH]),
            ops: NetDeviceOps {
                transmit: Loopback::transmit,
                open: NetDeviceOps::empty,
                close: NetDeviceOps::empty,
                poll: NetDeviceOps::empty,
            },
        }
    }

    pub fn init() {
        let loopback_dev = Loopback::new_device();
        NetDevice::register(loopback_dev);
    }
}
