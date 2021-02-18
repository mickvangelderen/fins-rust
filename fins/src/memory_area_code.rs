use crate::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct RawMemoryAreaCode(pub u8);

unsafe_impl_raw!(RawMemoryAreaCode);

impl RawMemoryAreaCode {
    pub const D: Self = Self(0x82);
}

pub enum MemoryAreaCode {
    D
}

impl MemoryAreaCode {
    pub const fn to_raw(&self) -> RawMemoryAreaCode {
        match self {
            MemoryAreaCode::D => RawMemoryAreaCode::D
        }
    }

    pub const fn from_raw(val: RawMemoryAreaCode) -> std::result::Result<Self, RawMemoryAreaCode> {
        match val {
            RawMemoryAreaCode::D => Ok(Self::D),
            unknown => Err(unknown)
        }
    }
}
