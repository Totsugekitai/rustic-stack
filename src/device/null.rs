use std::u16;

use crate::net::{
    NetDevice, NetDeviceAddress, NetDeviceOps, NetDeviceType, NetProtocolType, TransmitFnPtr,
    HARDWARE_ADDRESS_LENGTH,
};

const NULL_MTU: u16 = u16::MAX;

pub struct Null();

impl Null {
    pub fn transmit(
        dev: &NetDevice,
        net_device_type: u16,
        _data: *const u8,
        size: usize,
        _dst: *mut u8,
    ) -> isize {
        eprintln!(
            "DEV={} TYPE={} SIZE={}",
            dev.name,
            NetDeviceType::from_u16(net_device_type),
            size
        );
        0
    }

    pub fn new() -> NetDevice {
        NetDevice {
            name: String::from("null"),
            device_type: NetDeviceType::Null as u16 | NetProtocolType::Ip as u16,
            mtu: NULL_MTU,
            flags: 0,
            header_length: 0,
            address_length: 0,
            hwaddr: [0; HARDWARE_ADDRESS_LENGTH],
            pb: NetDeviceAddress::Peer([0; HARDWARE_ADDRESS_LENGTH]),
            ops: NetDeviceOps {
                transmit: Option::from(Null::transmit as TransmitFnPtr),
                open: None,
                close: None,
                poll: None,
            },
            interfaces: Vec::new(),
        }
    }

    pub fn init() {
        let null_dev = Null::new();
        NetDevice::register(null_dev);
    }
}
