use std::thread::sleep;
use std::time::Duration;

use rustic_stack::device::loopback::Loopback;
use rustic_stack::net::{net_run, net_shutdown, NetDeviceType, NetProtocolType, NET_DEVICES};

#[test]
fn loopback() {
    Loopback::init();

    let _ = net_run();

    for _ in 0..3 {
        let test_value = 0x32;
        const TEST_COUNT: usize = 8;
        let test_data: [u8; TEST_COUNT] = [test_value; TEST_COUNT];
        let mut test_dst: [u8; TEST_COUNT] = [0; TEST_COUNT];
        {
            let mut net_devices = NET_DEVICES.lock();
            for dev in net_devices.items.iter_mut() {
                if (dev.device_type & 0b11) == NetDeviceType::Loopback as u16 {
                    let r = dev.output(
                        NetDeviceType::Loopback as u16 & NetProtocolType::Ip as u16,
                        &test_data as *const [u8] as *const u8,
                        TEST_COUNT,
                        &mut test_dst as *mut [u8] as *mut u8,
                    );
                    match r {
                        Ok(_) => {
                            for i in test_dst {
                                if i != test_value {
                                    panic!("loopback is invalid! VALUE={}", i);
                                }
                            }
                        }
                        Err(_) => {
                            panic!("device output error");
                        }
                    }
                }
            }
        }

        sleep(Duration::from_secs(1));
    }
    let _ = net_shutdown();
}
