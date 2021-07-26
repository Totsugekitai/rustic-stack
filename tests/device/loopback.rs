use std::thread::sleep;
use std::time::Duration;

use rustic_stack::device::loopback::Loopback;
use rustic_stack::ipv4::IpInterface;
use rustic_stack::net::{
    net_init, net_run, net_shutdown, NetDeviceType, NetProtocolType, NET_DEVICES,
};

const LOOPBACK_IP_ADDRESS: &str = "127.0.0.1";
const LOOPBACK_IP_NETMASK: &str = "255.0.0.0";

#[test]
fn loopback() {
    net_init();

    let mut loopback_dev = Loopback::init();

    let interface = IpInterface::alloc(LOOPBACK_IP_ADDRESS, LOOPBACK_IP_NETMASK);
    if let None = interface {
        panic!("IpInterface::alloc is failed");
    }
    let interface = IpInterface::register(interface.unwrap(), &loopback_dev);

    let _ = net_run();

    for _ in 0..3 {
        let test_value = 0x32;
        const TEST_COUNT: usize = 8;
        let test_data: [u8; TEST_COUNT] = [test_value; TEST_COUNT];
        let mut test_dst: [u8; TEST_COUNT] = [0; TEST_COUNT];
        {
            let mut net_devices = NET_DEVICES.lock();
            for dev in net_devices.items.iter_mut() {
                if NetDeviceType::from_u16(dev.device_type) == NetDeviceType::Loopback {
                    let r = dev.output(
                        NetDeviceType::Loopback as u16 & NetProtocolType::Ip as u16,
                        &test_data as *const [u8] as *const u8,
                        TEST_COUNT,
                        &mut test_dst as *mut [u8] as *mut u8,
                    );
                    match r {
                        Ok(_) => {
                            for i in test_dst.iter() {
                                if *i != test_value {
                                    panic!("loopback is invalid! VALUE={}", i);
                                }
                            }
                            println!("loopback device output");
                        }
                        Err(_) => {
                            panic!("loopback device output error");
                        }
                    }
                }
            }
        }

        sleep(Duration::from_secs(1));
    }
    let _ = net_shutdown();
}
