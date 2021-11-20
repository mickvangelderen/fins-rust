use crate::*;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct RawMemoryAreaCode(pub u8);

unsafe_impl_raw!(RawMemoryAreaCode);

impl RawMemoryAreaCode {
    pub const D: Self = Self(0x82);

    pub const fn deserialize(self) -> Result<MemoryAreaCode, ProtocolViolation> {
        match self {
            RawMemoryAreaCode::D => Ok(MemoryAreaCode::D),
            unknown => Err(ProtocolViolation::InvalidMemoryAreaCode(unknown)),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
