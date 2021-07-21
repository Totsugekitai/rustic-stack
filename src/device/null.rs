use std::u16;

use crate::net::{NetDevice, NetDeviceOps, NetDeviceType};

const NULL_MTU: u16 = u16::MAX;

pub struct Null();

impl Null {
    pub fn new() -> Self {
        Null()
    }

    pub fn transmit(
        dev: &NetDevice,
        net_device_type: u16,
        _data: *const u8,
        size: usize,
        _dst: *const u8,
    ) -> isize {
        eprintln!(
            "DEV={} TYPE={} SIZE={}",
            dev.name,
            NetDeviceType::from_u16(net_device_type),
            size
        );
        0
    }

    pub fn setup(dev: &mut NetDevice) {
        dev.device_type = NetDeviceType::Null;
        dev.mtu = NULL_MTU;
        dev.header_length = 0;
        dev.address_length = 0;
        dev.ops.transmit = Null::transmit;
        dev.ops.open = NetDeviceOps::empty;
        dev.ops.close = NetDeviceOps::empty;
        dev.ops.poll = NetDeviceOps::empty;
    }

    pub fn init() {}
}
