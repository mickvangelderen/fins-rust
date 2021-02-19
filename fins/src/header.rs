use crate::*;

#[derive(Default)]
#[repr(C, packed)]
pub struct RawHeader {
    /// Information Control Field
    pub icf: RawInformationControlField,

    /// Reserved
    pub rsv: u8,

    /// Permissible number of gateways
    pub gct: u8,

    /// Destination address
    pub destination: RawMachineAddress,

    /// Source address.
    pub source: RawMachineAddress,

    /// Service ID
    /// Set by process to identify which one it came from.
    pub sid: u8,
}

unsafe_impl_raw!(RawHeader);

impl RawHeader {
    pub const fn deserialize(self) -> Result<Header> {
        Ok(Header {
            icf: trye!(self.icf.deserialize()),
            gct: self.gct,
            destination: self.destination.deserialize(),
            source: self.source.deserialize(),
            sid: self.sid,
        })
    }
}

#[derive(Debug)]
pub struct Header {
    pub icf: InformationControlField,

    pub gct: u8,

    pub destination: MachineAddress,

    pub source: MachineAddress,

    pub sid: u8,
}

impl Header {
    pub const fn serialize(&self) -> RawHeader {
        RawHeader {
            icf: self.icf.serialize(),
            rsv: 0x00,
            gct: self.gct,
            destination: self.destination.serialize(),
            source: self.source.serialize(),
            sid: self.sid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_to_bytes_works() {
        let input = Header {
            icf: InformationControlField::RequestWithResponse,
            gct: 0x02,
            destination: MachineAddress {
                network: 0x03,
                node: 0x04,
                unit: 0x05,
            },
            source: MachineAddress {
                network: 0x06,
                node: 0x07,
                unit: 0x08,
            },
            sid: 0x09,
        };

        assert_eq!(
            input.serialize().bytes(),
            &[0x80, 0x00, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,]
        );
    }
}
