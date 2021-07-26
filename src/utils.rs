pub fn checksum16(address: *const u16, header_count: u16, init: u32) -> u16 {
    let mut sum = init as u32;
    let mut count = header_count;
    let mut address = address;

    while count > 1 {
        sum += unsafe { *address as u32 };
        address = ((address as usize) + std::mem::size_of::<u16>()) as *const u16;
        count -= 2;
    }
    if count > 0 {
        sum += unsafe { *(address as *const u8) as u32 };
    }

    while (sum >> 16) > 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }

    !(sum as u16)
}
