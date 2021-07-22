use std::mem;
use std::thread::sleep;
use std::time::Duration;

use rustic_stack::device::null::Null;
use rustic_stack::net::{net_run, net_shutdown, NET_DEVICES};

#[test]
fn null() {
    Null::init();

    let _ = net_run();

    let test_data: [u8; 8] = [0; 8];
    let test_dst: [u8; 8] = [0; 8];

    for _ in 0..3 {
        {
            let mut net_devices = NET_DEVICES.lock();
            for dev in net_devices.items.iter_mut() {
                if dev.name == "null" {
                    let r = dev.output(
                        0x0800,
                        &test_data as *const [u8] as *const u8,
                        mem::size_of::<[u8; 0]>(),
                        &test_dst as *const [u8] as *mut u8,
                    );
                    match r {
                        Ok(_) => (),
                        Err(_) => {
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        }

        sleep(Duration::from_secs(1));
    }
    let _ = net_shutdown();
}
