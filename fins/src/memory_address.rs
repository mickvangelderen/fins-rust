use crate::*;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[repr(C, packed)]
pub struct RawMemoryAddress {
    area_code: RawMemoryAreaCode,
    offset: u16be,
    bits: u8,
}

impl RawMemoryAddress {
    pub const fn deserialize(self) -> Result<MemoryAddress> {
        let RawMemoryAddress { area_code, offset, bits } = self;
        Ok(MemoryAddress {area_code: trye!(area_code.deserialize()), offset: offset.to_u16(), bits })
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MemoryAddress {
    pub area_code: MemoryAreaCode,
    pub offset: u16,
    pub bits: u8,
}

impl MemoryAddress {
    pub const fn serialize(&self) -> RawMemoryAddress {
        RawMemoryAddress {
            area_code: self.area_code.serialize(),
            offset: u16be::from_u16(self.offset),
            bits: self.bits,
        }
    }
}

impl std::fmt::Debug for MemoryAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.bits == 0 {
            write!(f, "{:?}{}", self.area_code, self.offset)
        } else {
            write!(f, "{:?}{}.{}", self.area_code, self.offset, self.bits)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_works() {
        assert_eq!(
            MemoryAddress { area_code: MemoryAreaCode::D, offset: 100, bits: 0 }.serialize(),
            RawMemoryAddress { area_code: RawMemoryAreaCode::D, offset: u16be::from_u16(100), bits: 0 }
        );
    }

    #[test]
    fn layout_is_nice() {
        assert_eq!(std::mem::size_of::<MemoryAddress>(), 4);
    }
}
