use crate::*;

#[derive(Default)]
#[repr(C, packed)]
pub struct RawMemoryAddress(u32be);

#[derive(Debug)]
pub struct MemoryAddress(u32);

impl MemoryAddress {
    pub const fn new(area_code: MemoryAreaCode, offset: u32) -> Self {
        let ac = (area_code.to_raw().0 as u32) << 24;
        let of = offset & 0xFFFFFF00;
        Self(ac | of)
    }
    
    pub fn area_code(&self) -> MemoryAreaCode {
        // Unwrap should never fail because we perform this check in `from_raw`.
        MemoryAreaCode::from_raw(RawMemoryAreaCode((self.0 >> 24) as u8)).unwrap()
    }

    pub const fn offset(&self) -> u32 {
        self.0 & 0xFFFFFF00
    }

    pub const fn to_raw(&self) -> RawMemoryAddress {
        RawMemoryAddress(u32be::from_u32(self.0))
    }

    pub const fn from_raw(val: RawMemoryAddress) -> std::result::Result<Self, Error> {
        let val = val.0.to_u32();
        if let Err(e) = MemoryAreaCode::from_raw(RawMemoryAreaCode((val >> 24) as u8)) {
            return Err(Error::InvalidMemoryAddressCode(e));
        }
        Ok(Self(val))
    }
}
