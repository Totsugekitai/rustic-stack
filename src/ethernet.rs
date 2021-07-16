pub const ADDRESS_LENGTH: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddress([u8; ADDRESS_LENGTH]);
