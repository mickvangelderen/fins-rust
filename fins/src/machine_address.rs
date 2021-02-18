use crate::*;

#[derive(Default)]
#[repr(C, packed)]
pub struct RawMachineAddress {
    pub network: u8,
    pub node: u8,
    pub unit: u8,
}

unsafe_impl_raw!(RawMachineAddress);

#[derive(Debug)]
pub struct MachineAddress {
    pub network: u8,
    pub node: u8,
    pub unit: u8,
}

impl MachineAddress {
    pub const fn to_raw(&self) -> RawMachineAddress {
        let &Self { network, node, unit } = self;
        RawMachineAddress { network, node, unit }
    }

    pub const fn from_raw(raw: RawMachineAddress) -> Self {
        let RawMachineAddress { network, node, unit } = raw;
        Self { network, node, unit }
    }
}
