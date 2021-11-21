mod client_address_frame;
mod command_code;
mod error;
mod header;
mod protocol_violation;
mod server_address_frame;

use std::io::{Read, Write};

pub use client_address_frame::*;
pub use command_code::*;
pub use error::*;
use fins::{MachineAddress, MemoryAreaReadRequest};
use fins_util::ReadExt;
pub use header::*;
pub use protocol_violation::*;
pub use server_address_frame::*;

pub fn write_memory_area_read_request<W: Write>(
    writer: &mut W,
    request: &MemoryAreaReadRequest,
) -> crate::Result<()> {
    Header {
        command: CommandCode::Fins,
        length: 8 + MemoryAreaReadRequest::byte_size() as u32,
        error_code: 0,
    }
    .write_to(writer)?;

    request.write_to(writer)?;

    Ok(())
}

pub struct MemoryAreaReadResponse {
    pub src_addr: MachineAddress,
    pub dst_addr: MachineAddress,
    pub bytes: Vec<u8>,
    pub service_id: u8,
}

pub fn read_memory_area_read_response<R: Read>(
    reader: &mut R,
) -> crate::Result<MemoryAreaReadResponse> {
    let Header {
        length,
        command,
        error_code,
    } = Header::read_from(reader)?;

    assert_command(command, CommandCode::Fins)?;
    assert_no_error(error_code)?;

    let fins::Header {
        icf,
        gct: _,
        destination,
        source,
        sid,
    } = reader.read_raw::<fins::RawHeader>()?.deserialize()?;

    assert_eq!(icf, fins::InformationControlField::ResponseWithResponse);
    // assert_eq!(
    //     destination,
    //     fins::MachineAddress {
    //         network: 0,
    //         node: self.client_node,
    //         unit: 0
    //     }
    // );
    // assert_eq!(
    //     source,
    //     fins::MachineAddress {
    //         network: 0,
    //         node: self.server_node,
    //         unit: 0
    //     }
    // );

    let fins::RawResponseHeader {
        mrc,
        src,
        mres,
        sres,
    } = reader.read_raw::<fins::RawResponseHeader>()?;

    assert_eq!([mrc, src], [0x01, 0x01]);
    assert_eq!([mres, sres], [0x00, 0x00]);

    let byte_count = length
        - (std::mem::size_of::<RawHeader>() as u32 - 8)
        - std::mem::size_of::<fins::RawHeader>() as u32
        - std::mem::size_of::<fins::RawResponseHeader>() as u32;

    let mut bytes = Vec::with_capacity(byte_count as usize);
    bytes.resize(byte_count as usize, 0);
    reader.read_exact(&mut bytes[..])?;

    Ok(MemoryAreaReadResponse {
        src_addr: source,
        dst_addr: destination,
        bytes,
        service_id: sid,
    })
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use fins::{MemoryAddress, MemoryAreaCode};

    use super::*;

    #[test]
    fn works() {
        let mut buffer = vec![];
        let mut cursor = Cursor::new(&mut buffer);
        write_memory_area_read_request(
            &mut cursor,
            &MemoryAreaReadRequest {
                server_node: 0xD3,
                client_node: 0xFB,
                address: MemoryAddress {
                    area_code: MemoryAreaCode::D,
                    offset: 1500,
                    bits: 0,
                },
                count: 16,
            },
        )
        .unwrap();

        assert_eq!(
            &buffer[..],
            &[
                0x46, 0x49, 0x4E, 0x53, // FINS
                0x00, 0x00, 0x00, 0x1A, // length: 26
                0x00, 0x00, 0x00, 0x02, // command: fins
                0x00, 0x00, 0x00, 0x00, // error: none
                0x80, // ICF: use gateway, command with response
                0x00, // RSV
                0x02, // GCT: gateway count 2
                0x00, 0xD3, 0x00, // src addr
                0x00, 0xFB, 0x00, // dst addr
                0x00, // SID
                0x01, 0x01, // request code: memory area read
                0x82, 0x05, 0xDC, 0x00, // memory address: location D, offset 1500, bit 0
                0x00, 0x10, // word count: 16
            ]
        )
    }
}
