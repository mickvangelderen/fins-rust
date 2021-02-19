use crate::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct RawMemoryAreaCode(pub u8);

unsafe_impl_raw!(RawMemoryAreaCode);

impl RawMemoryAreaCode {
    pub const D: Self = Self(0x82);

    pub const fn deserialize(self) -> Result<MemoryAreaCode> {
        match self {
            RawMemoryAreaCode::D => Ok(MemoryAreaCode::D),
            unknown => Err(Error::InvalidMemoryAddressCode(unknown)),
        }
    }
}

#[derive(Debug)]
pub enum MemoryAreaCode {
    D,
}

impl MemoryAreaCode {
    pub const fn serialize(&self) -> RawMemoryAreaCode {
        match self {
            MemoryAreaCode::D => RawMemoryAreaCode::D,
        }
    }
}
