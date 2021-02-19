use crate::*;

#[derive(Default)]
#[repr(C, packed)]
pub struct RawMachineAddress {
    pub network: u8,
    pub node: u8,
    pub unit: u8,
}

unsafe_impl_raw!(RawMachineAddress);

impl RawMachineAddress {
    pub const fn deserialize(self) -> MachineAddress {
        let Self {
            network,
            node,
            unit,
        } = self;
        MachineAddress {
            network,
            node,
            unit,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct MachineAddress {
    pub network: u8,
    pub node: u8,
    pub unit: u8,
}

impl MachineAddress {
    pub const fn serialize(&self) -> RawMachineAddress {
        let &Self {
            network,
            node,
            unit,
        } = self;
        RawMachineAddress {
            network,
            node,
            unit,
        }
    }
}
