use crate::*;

#[derive(Default)]
#[repr(transparent)]
pub struct RawMemoryAddress(u32be);

impl RawMemoryAddress {
    pub const fn deserialize(self) -> Result<MemoryAddress> {
        let val = self.0.to_u32();
        trye!(RawMemoryAreaCode((val >> 24) as u8).deserialize());
        Ok(MemoryAddress(val))
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct MemoryAddress(u32);

impl MemoryAddress {
    pub const fn new(area_code: MemoryAreaCode, offset: u16, bits: u8) -> Self {
        Self(
            bits as u32
            | (offset as u32) << 8
            | (area_code.serialize().0 as u32) << 24
        )
    }

    pub fn area_code(&self) -> MemoryAreaCode {
        // Unwrap should never fail because we perform this check in `deserialize>>>`.
        RawMemoryAreaCode((self.0 >> 24) as u8)
            .deserialize()
            .unwrap()
    }

    pub const fn offset(&self) -> u16 {
        (self.0 >> 8) as u16
    }

    pub const fn bits (&self) -> u8 {
        self.0 as u8
    }

    pub const fn serialize(&self) -> RawMemoryAddress {
        RawMemoryAddress(u32be::from_u32(self.0))
    }
}

impl std::fmt::Debug for MemoryAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}{}", self.area_code(), self.offset())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_works() {
        assert_eq!(MemoryAddress::new(MemoryAreaCode::D, 100, 0).serialize().0, u32be::from_bytes([0x80, 0x64, 0x00, 0x00]))
    }
}