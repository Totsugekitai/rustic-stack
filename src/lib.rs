pub mod device;
pub mod ethernet;
pub mod ipv4;
pub mod packet;

mod tests {
    #[test]
    fn device() {
        if let Ok(mac_addresses) = crate::device::get_mac_address_list() {
            for mac_address in mac_addresses {
                eprintln!("{}", mac_address);
            }
        }
    }
    #[test]
    fn ethernet() {}
}
