use std::mem;
use std::thread::sleep;
use std::time::Duration;

use rustic_stack::device::loopback::Loopback;
use rustic_stack::net::{net_run, net_shutdown, NET_DEVICES};

#[test]
fn loopback() {
    Loopback::init();

    let _ = net_run();

    for _ in 0..10 {
        let test_data: [u8; 8] = [0x32; 8];
        let mut test_dst: [u8; 8] = [0; 8];
        {
            let mut net_devices = NET_DEVICES.lock();
            for dev in net_devices.items.iter_mut() {
                if dev.name == "loopback" {
                    let r = dev.output(
                        0x0800,
                        &test_data as *const [u8] as *const u8,
                        mem::size_of::<[u8; 0]>(),
                        &mut test_dst as *mut [u8] as *mut u8,
                    );
                    match r {
                        Ok(_) => (),
                        Err(_) => {
                            panic!("device output error");
                        }
                    }
                } else {
                    panic!("device not found");
                }
            }
        }
        for i in test_dst {
            if i != 0x32 {
                panic!("loopback is invalid!");
            }
        }

        sleep(Duration::from_secs(1));
    }
    let _ = net_shutdown();
}
