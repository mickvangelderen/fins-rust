mod client_address_frame;
mod command_code;
mod error;
mod header;
mod protocol_violation;
mod server_address_frame;

pub use client_address_frame::*;
pub use command_code::*;
pub use error::*;
pub use header::*;
pub use protocol_violation::*;
pub use server_address_frame::*;

pub struct State {
    pub client_node: u8,
    pub server_node: u8,
}
// impl State {
//     pub fn read_frame(&mut self, address: fins::MemoryAddress, count: u16) -> Result<Vec<u8>> {
//         {
//             #[derive(Default)]
//             #[repr(C, packed)]
//             struct RawRequest {
//                 fins_tcp_header: RawHeader,
//                 fins_header: fins::RawHeader,
//                 fins_request: fins::RawRequestHeader,
//                 address: fins::RawMemoryAddress,
//                 count: u16be,
//             }
//             unsafe_impl_raw!(RawRequest);

//             let request = RawRequest {
//                 fins_tcp_header: Header {
//                     command: CommandCode::Fins,
//                     length: std::mem::size_of::<RawRequest>() as u32 - 8,
//                     error_code: 0,
//                 }
//                 .to_raw(),
//                 fins_header: fins::Header {
//                     icf: fins::InformationControlField::RequestWithResponse,
//                     gct: 0x02,
//                     destination: fins::MachineAddress {
//                         network: 0,
//                         node: self.server_node,
//                         unit: 0,
//                     },
//                     source: fins::MachineAddress {
//                         network: 0,
//                         node: self.client_node,
//                         unit: 0,
//                     },
//                     sid: 0,
//                 }
//                 .serialize(),
//                 fins_request: fins::RawRequestHeader {
//                     mrc: 0x01,
//                     src: 0x01,
//                 },
//                 address: address.serialize(),
//                 count: u16be::from_u16(count),
//             };

//             write_raw!(&mut self.stream, request);
//             self.stream.flush().await?;
//         }

//         {
//             let Header {
//                 length,
//                 command,
//                 error_code,
//             } = Header::from_raw(read_raw!(&mut self.stream, RawHeader))?;

//             assert_eq!(command, CommandCode::Fins);
//             assert_eq!(0, error_code);

//             let fins::Header {
//                 icf,
//                 gct: _,
//                 destination,
//                 source,
//                 sid,
//             } = read_raw!(&mut self.stream, fins::RawHeader).deserialize()?;

//             assert_eq!(icf, InformationControlField::ResponseWithResponse);
//             assert_eq!(
//                 destination,
//                 fins::MachineAddress {
//                     network: 0,
//                     node: self.client_node,
//                     unit: 0
//                 }
//             );
//             assert_eq!(
//                 source,
//                 fins::MachineAddress {
//                     network: 0,
//                     node: self.server_node,
//                     unit: 0
//                 }
//             );
//             assert_eq!(sid, 0); // whatever?

//             let fins::RawResponseHeader {
//                 mrc,
//                 src,
//                 mres,
//                 sres,
//             } = read_raw!(&mut self.stream, fins::RawResponseHeader);

//             assert_eq!(mrc, 1);
//             assert_eq!(src, 1);
//             assert_eq!(mres, 0);
//             assert_eq!(sres, 0);

//             let byte_count = length
//                 - (std::mem::size_of::<RawHeader>() as u32 - 8)
//                 - std::mem::size_of::<fins::RawHeader>() as u32
//                 - std::mem::size_of::<fins::RawResponseHeader>() as u32;

//             let mut bytes = Vec::with_capacity(byte_count as usize);
//             bytes.resize(byte_count as usize, 0);
//             self.stream.read_exact(&mut bytes[..]).await?;

//             Ok(bytes)
//         }
//     }
// }

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
