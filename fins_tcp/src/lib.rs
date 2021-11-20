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
use fins_util::{ReadExt, WriteExt};
pub use header::*;
pub use protocol_violation::*;
pub use server_address_frame::*;

pub fn write_memory_area_read_request<W: Write>(
    writer: &mut W,
    request: &MemoryAreaReadRequest,
) -> crate::Result<()> {
    Header {
        command: CommandCode::Fins,
        length: 4 + MemoryAreaReadRequest::byte_size() as u32,
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
    assert_eq!(sid, 0); // whatever?

    let fins::RawResponseHeader {
        mrc,
        src,
        mres,
        sres,
    } = reader.read_raw::<fins::RawResponseHeader>()?;

    assert_eq!(mrc, 1);
    assert_eq!(src, 1);
    assert_eq!(mres, 0);
    assert_eq!(sres, 0);

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
    })
}

#[derive(Debug)]
pub struct FinsRequestFrame {}

#[derive(Debug)]
pub enum FinsResponseFrame {
    Read(Vec<u8>),
}

#[derive(Debug)]
pub enum FinsFrame {
    Request(FinsRequestFrame),
    Response(FinsResponseFrame),
}

// impl FinsFrame {
//     pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
//         // let fins::Header {
//         //     icf,
//         //     gct,
//         //     destination,
//         //     source,
//         //     sid,
//         // } = read_raw!(reader, fins::RawHeader).deserialize()?;

//         unimplemented!()

//         // if icf.is_request() {
//         //     let fins::RawRequestHeader { mrc, src } = read_raw!(reader, fins::RawRequestHeader);
//         //     unimplemented!()
//         // } else {
//         //     let fins::RawResponseHeader { mrc, src, mres, sres } = read_raw!(reader, fins::RawResponseHeader);
//         //     assert_eq!(mrc, 1);
//         //     assert_eq!(src, 1);
//         //     assert_eq!(mres, 0);
//         //     assert_eq!(sres, 0);
//         //     Ok(FinsFrame::Response(
//         //         FinsResponseFrame::Read()
//         //     ))
//         // }
//     }
// }

// #[derive(Debug)]
// pub enum FinsTcpFrame {
//     ClientAddress(ClientAddressFrame),
//     ServerAddress(ServerAddressFrame),
//     Fins(FinsFrame),
// }

// impl FinsTcpFrame {
//     pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
//         let Header {
//             length,
//             command,
//             error_code,
//         } = Header::from_raw(read_raw!(reader, RawHeader))?;

//         // FIXME: Figure out what is sent on error. We don't want to break the stream by not reading the entire frame!
//         if error_code != 0 {
//             return Err(Error::ErrorCode {
//                 command,
//                 error_code,
//             });
//         }

//         Ok(match command {
//             CommandCode::ClientAddress => {
//                 if length != CLIENT_ADDRESS_LENGTH {
//                     return Err(Error::Invalid);
//                 };
//                 let body = read_raw!(reader, RawClientAddressBody);
//                 FinsTcpFrame::ClientAddress(ClientAddressFrame::from_raw_body(body))
//             }
//             CommandCode::ServerAddress => {
//                 if length != SERVER_ADDRESS_LENGTH {
//                     return Err(Error::Invalid);
//                 };
//                 let body = read_raw!(reader, RawServerAddressBody);
//                 FinsTcpFrame::ServerAddress(ServerAddressFrame::from_raw_body(body))
//             }
//             CommandCode::Fins => unimplemented!(),
//         })
//     }
// }
