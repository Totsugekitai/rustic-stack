use std::{fs, io, path};

fn read_mac_address(address_file: &path::Path) -> Result<String, io::Error> {
    let mac_address = fs::read_to_string(address_file)?;
    Ok(mac_address)
}

pub fn get_mac_address_list() -> Result<Vec<String>, io::Error> {
    let net_devices_root_dir = path::Path::new("/sys/class/net");
    let entries = fs::read_dir(net_devices_root_dir)?;
    let mut address_vec = Vec::new();
    for entry in entries {
        let entry = entry?;
        // let metadata = entry.metadata()?;
        // let path = entry.path().display().to_string();

        if let Ok(symlink) = fs::read_link(entry.path()) {
            let path = symlink.as_path();
            let address_file_path = path.join("address");
            let mac_address = read_mac_address(&address_file_path)?;
            address_vec.push(mac_address);
        }
    }
    Ok(address_vec)
}
