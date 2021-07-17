use std::fmt;

pub struct Ipv4(u32);

impl fmt::Display for Ipv4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.0 >> 24,
            (self.0 >> 16) & 0xff,
            (self.0 >> 8) & 0xff,
            self.0 & 0xff
        )
    }
}

#[repr(C, packed)]
pub struct Ipv4Packet {
    version_and_ihl: u8,
    dscp_and_ecn: u8,
    total_length: u16,
    id: u16,
    flags_and_flagment_offset: u16,
    time_to_live: u8,
    protocol: u8,
    header_checksum: u16,
    src_ip_address: u32,
    dst_ip_address: u32,
    option_and_padding: u32,
    payload: Vec<u8>,
}

impl Ipv4Packet {
    pub fn version(&self) -> u8 {
        self.version_and_ihl & 0b1111
    }

    pub fn header_length(&self) -> u8 {
        (self.version_and_ihl >> 4) & 0b1111
    }

    pub fn dscp(&self) -> u8 {
        self.dscp_and_ecn & 0b111111
    }

    pub fn ecn(&self) -> u8 {
        (self.dscp_and_ecn >> 6) & 0b11
    }

    pub fn packet_length(&self) -> u16 {
        self.total_length
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn flags(&self) -> u16 {
        self.flags_and_flagment_offset & 0b111
    }

    pub fn fragment_offset(&self) -> u16 {
        self.flags_and_flagment_offset >> 3
    }

    pub fn time_to_live(&self) -> u8 {
        self.time_to_live
    }

    pub fn protocol(&self) -> Protocol {
        match self.protocol {
            1 => Protocol::Icmp,
            4 => Protocol::Ip,
            6 => Protocol::Tcp,
            17 => Protocol::Udp,
            _ => Protocol::Unimplement,
        }
    }

    pub fn src_address(&self) -> Ipv4 {
        Ipv4(self.src_ip_address)
    }

    pub fn dst_address(&self) -> Ipv4 {
        Ipv4(self.dst_ip_address)
    }
}

#[repr(u8)]
pub enum Protocol {
    Icmp = 1,
    Ip = 4,
    Tcp = 6,
    Udp = 17,
    Unimplement,
}

pub fn handle(_packet: &Ipv4Packet) {}
