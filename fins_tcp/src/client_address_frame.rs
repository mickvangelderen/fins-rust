use fins_util::{u32be, unsafe_impl_raw, ReadExt, WriteExt};
use std::{
    convert::TryInto,
    io::{Read, Write},
};

use crate::{
    assert_command, assert_header_length, assert_no_error, CommandCode, Header, RawHeader, Result,
};

#[derive(Debug, Eq, PartialEq)]
pub struct ClientAddressFrame {
    pub client_node: u8,
}

const CLIENT_ADDRESS_LENGTH: u32 = 12;

impl ClientAddressFrame {
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        Header {
            length: CLIENT_ADDRESS_LENGTH,
            command: CommandCode::ClientAddress,
            error_code: 0,
        }
        .write_to(writer)?;
        writer.write_raw(&RawClientAddressBody {
            client_node: u32be::from(self.client_node as u32),
        })?;
        Ok(())
    }

    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let Header {
            length,
            command,
            error_code,
        } = Header::read_from(reader)?;

        assert_header_length(length, CLIENT_ADDRESS_LENGTH)?;
        assert_command(command, CommandCode::ClientAddress)?;
        assert_no_error(error_code)?;

        let RawClientAddressBody { client_node } = reader.read_raw::<RawClientAddressBody>()?;

        Ok(Self {
            client_node: client_node.to_u32().try_into().unwrap(),
        })
    }
}

#[derive(Default)]
#[repr(C, packed)]
struct RawClientAddressFrame {
    header: RawHeader,
    body: RawClientAddressBody,
}

unsafe_impl_raw!(RawClientAddressFrame);

#[derive(Default)]
#[repr(C, packed)]
struct RawClientAddressBody {
    client_node: u32be,
}

unsafe_impl_raw!(RawClientAddressBody);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let input = ClientAddressFrame { client_node: 3 };

        let mut output = vec![];
        input
            .write_to(&mut std::io::Cursor::new(&mut output))
            .unwrap();

        assert_eq!(
            output,
            [
                0x46, 0x49, 0x4E, 0x53, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
            ]
        );

        assert_eq!(
            ClientAddressFrame::read_from(&mut std::io::Cursor::new(&output)).unwrap(),
            input
        );
    }
}
