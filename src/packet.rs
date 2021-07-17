pub trait Packet {
    fn payload(&self) -> &Vec<u8>;
}
