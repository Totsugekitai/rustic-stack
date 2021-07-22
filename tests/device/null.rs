use std::thread::sleep;
use std::time::Duration;

use rustic_stack::device::null::Null;
use rustic_stack::net::{
    net_init, net_run, net_shutdown, NetDeviceType, NetProtocolType, NET_DEVICES,
};

#[test]
fn null() {
    net_init();

    Null::init();

    let _ = net_run();

    for _ in 0..3 {
        const TEST_COUNT: usize = 8;
        let test_data: [u8; TEST_COUNT] = [0; TEST_COUNT];
        let test_dst: [u8; TEST_COUNT] = [0; TEST_COUNT];

        {
            let mut net_devices = NET_DEVICES.lock();
            for dev in net_devices.items.iter_mut() {
                if NetDeviceType::from_u16(dev.device_type) == NetDeviceType::Null {
                    let r = dev.output(
                        NetDeviceType::Null as u16 & NetProtocolType::Ip as u16,
                        &test_data as *const [u8] as *const u8,
                        TEST_COUNT,
                        &test_dst as *const [u8] as *mut u8,
                    );
                    match r {
                        Ok(_) => {
                            println!("null device output");
                        }
                        Err(_) => {
                            panic!("null device output error");
                        }
                    }
                }
            }
        }

        sleep(Duration::from_secs(1));
    }
    let _ = net_shutdown();
}
