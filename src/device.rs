pub mod linux {
    use once_cell::sync::OnceCell;
    use std::fs::{read_dir, read_link, read_to_string, File};
    use std::io;
    use std::path::Path;

    static TAP_FILE: OnceCell<File> = OnceCell::new();

    fn read_mac_address(address_file: &Path) -> Result<String, io::Error> {
        let mac_address = read_to_string(address_file)?;
        Ok(mac_address)
    }

    pub fn get_mac_address_list() -> Result<Vec<String>, io::Error> {
        let net_devices_root_dir = Path::new("/sys/class/net");
        let entries = read_dir(net_devices_root_dir)?;
        let mut address_vec = Vec::new();
        for entry in entries {
            let entry = entry?;
            // let metadata = entry.metadata()?;
            // let path = entry.path().display().to_string();

            if let Ok(symlink) = read_link(entry.path()) {
                let path = symlink.as_path();
                let address_file_path = path.join("address");
                let mac_address = read_mac_address(&address_file_path)?;
                address_vec.push(mac_address);
            }
        }
        Ok(address_vec)
    }

    pub fn open_tap() -> io::Result<()> {
        let tap_path = Path::new("/dev/net/tap");
        let tap_file = File::open(tap_path)?;
        TAP_FILE.set(tap_file).unwrap();
        Ok(())
    }

    pub fn poll_tap() {}
    pub fn read_tap() {}
    pub fn write_tap() {}
    pub fn close_tap() {}
}
