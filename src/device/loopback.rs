use std::ptr;
use std::u16;

use crate::net::{
    NetDevice, NetDeviceAddress, NetDeviceFlag, NetDeviceOps, NetDeviceType, NetProtocolType,
    TransmitFnPtr, HARDWARE_ADDRESS_LENGTH,
};

const LOOPBACK_MTU: u16 = u16::MAX;

pub struct Loopback();

impl Loopback {
    pub fn transmit(
        dev: &NetDevice,
        protocol_type: u16,
        data: *const u8,
        size: usize,
        dst: *mut u8,
    ) -> isize {
        println!(
            "DEV={} PROTOCOL_TYPE={:04x} SIZE={}",
            dev.name, protocol_type, size
        );
        unsafe { ptr::copy_nonoverlapping(data, dst, size) };
        for i in 0..size {
            // コピーの検査
            unsafe {
                if *((data as usize + i) as *const u8) != *((dst as usize + i) as *const u8) {
                    return -1;
                }
            }
        }
        size as isize
    }

    pub fn new() -> NetDevice {
        NetDevice {
            name: String::from("loopback"),
            device_type: NetDeviceType::Loopback as u16 | NetProtocolType::Ip as u16,
            mtu: LOOPBACK_MTU,
            flags: NetDeviceFlag::Loopback as u16,
            header_length: 0,
            address_length: 0,
            hwaddr: [0; HARDWARE_ADDRESS_LENGTH],
            pb: NetDeviceAddress::Peer([0; HARDWARE_ADDRESS_LENGTH]),
            ops: NetDeviceOps {
                transmit: Option::from(Loopback::transmit as TransmitFnPtr),
                open: None,
                close: None,
                poll: None,
            },
        }
    }

    pub fn init() {
        let loopback_dev = Loopback::new();
        NetDevice::register(loopback_dev);
    }
}
