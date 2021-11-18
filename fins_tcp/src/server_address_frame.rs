use std::{convert::TryInto, io::{Read, Write}};

use fins_util::{ReadExt, WriteExt, u32be, unsafe_impl_raw};

use crate::{assert_command, assert_header_length, assert_no_error, CommandCode, Header};

const SERVER_ADDRESS_LENGTH: u32 = 16;

#[derive(Debug, Eq, PartialEq)]
pub struct ServerAddressFrame {
    pub client_node: u8,
    pub server_node: u8,
}

impl ServerAddressFrame {
    pub fn read_from<R: Read>(reader: &mut R) -> crate::Result<Self> {
        let Header {
            length,
            command,
            error_code,
        } = Header::read_from(reader)?;

        assert_header_length(length, SERVER_ADDRESS_LENGTH)?;
        assert_command(command, CommandCode::ServerAddress)?;
        assert_no_error(error_code)?;

        let body = reader.read_raw::<RawServerAddressBody>()?;

        Ok(Self::from_raw_body(body))
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        Header {
            length: SERVER_ADDRESS_LENGTH,
            command: CommandCode::ServerAddress,
            error_code: 0,
        }.write_to(writer)?;
        writer.write_raw(&RawServerAddressBody {
            client_node: u32be::from_u32(self.client_node as u32),
            server_node: u32be::from_u32(self.server_node as u32),
        })?;
        Ok(())
    }
    
    fn from_raw_body(body: RawServerAddressBody) -> Self {
        Self {
            client_node: body.client_node.to_u32().try_into().unwrap(),
            server_node: body.server_node.to_u32().try_into().unwrap(),
        }
    }
}

#[derive(Default)]
#[repr(C, packed)]
struct RawServerAddressBody {
    client_node: u32be,
    server_node: u32be,
}

unsafe_impl_raw!(RawServerAddressBody);

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn round_trip() {
        let input = ServerAddressFrame {
            client_node: 3,
            server_node: 4,
        };

        let mut output = vec![];
        input.write_to(&mut Cursor::new(&mut output)).unwrap();

        assert_eq!(
            output,
            [
                0x46, 0x49, 0x4E, 0x53, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04,
            ]
        );

        assert_eq!(
            ServerAddressFrame::read_from(&mut Cursor::new(&output)).unwrap(),
            input
        );
    }
}
