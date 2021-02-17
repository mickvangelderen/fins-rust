mod information_control_field;

pub use information_control_field::*;

use std::io::Result;
use tokio::io::{AsyncWrite, AsyncWriteExt, AsyncRead, AsyncReadExt};
use fins_util::*;

pub struct RequestFrame {
    pub client_node: u8,
    pub server_node: u8,
    pub service_id: u8,
    pub mrc: u8,
    pub src: u8,
    pub body: Vec<u8>
}

impl RequestFrame {
    pub async fn write_to<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        RawRequestHeader::from(self).write_to(writer).await?;
        writer.write_all(&self.body).await?;
        Ok(())
    }
}

impl From<&RequestFrame> for RawRequestHeader {
    fn from(val: &RequestFrame) -> Self {
        Self {
            icf: InformationControlField::RequestWithResponse.into(),
            rsv: 0x00,
            gct: 0x02,
            dna: 0x00,
            da1: val.server_node,
            da2: 0x00,
            sna: 0x00,
            sa1: val.client_node,
            sa2: 0x00,
            sid: val.service_id,
            mrc: val.mrc,
            src: val.src,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
struct RawRequestHeader {
    /// Information Control Field
    pub icf: RawInformationControlField,

    /// Reserved
    pub rsv: u8,

    /// Permissible number of gateways
    pub gct: u8,

    /// Destination Network Address
    pub dna: u8,

    /// Destination Node Address
    pub da1: u8,

    /// Destination Unit Address
    pub da2: u8,

    /// Source Network Address
    pub sna: u8,

    /// Source Node Address
    pub sa1: u8,

    /// Source Unit Address
    pub sa2: u8,

    /// Service ID
    /// Set by process to identify which one it came from.
    pub sid: u8,

    /// Main Request Code
    pub mrc: u8,

    /// Sub Request Code
    pub src: u8,
}

unsafe_impl_raw!(RawRequestHeader);

#[derive(Debug)]
#[repr(C)]
struct RawResponseHeader {
    /// Information Control Field
    pub icf: RawInformationControlField,

    /// Reserved
    pub rsv: u8,

    /// Permissible number of gateways
    pub gct: u8,

    /// Destination Network Address
    pub dna: u8,

    /// Destination Node Address
    pub da1: u8,

    /// Destination Unit Address
    pub da2: u8,

    /// Source Network Address
    pub sna: u8,

    /// Source Node Address
    pub sa1: u8,

    /// Source Unit Address
    pub sa2: u8,

    /// Service ID
    /// Set by process to identify which one it came from.
    pub sid: u8,

    /// Main Request Code
    pub mrc: u8,

    /// Sub Request Code
    pub src: u8,

    /// Main Response Code
    pub mres: u8,

    /// Sub Repsonse Code
    pub sres: u8,
}

unsafe_impl_raw!(RawResponseHeader);


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn request_frame_serializes() {
        let mut output = [0u8; 18];

        RequestFrame {
            client_node: 1,
            server_node: 2,
            service_id: 3,
            mrc: 4,
            src: 5,
            body: vec![
                0x82,
                0x00,
                0x64,
                0x00,
                0x00,
                0x96,
            ],
        }.write_to(
            &mut std::io::Cursor::new(&mut output[..])
        ).await.unwrap();

        assert_eq!(output, [
            0x80,
            0x00,
            0x02,
            0x00,
            0x02,
            0x00,
            0x00,
            0x01,
            0x00,
            0x03,
            0x04,
            0x05,
            0x82,
            0x00,
            0x64,
            0x00,
            0x00,
            0x96,
        ]);
    }

    // #[tokio::test]
    // async fn connect_response_deserializes() {
    //     let input = [
    //         0x46, 0x49, 0x4E, 0x53,
    //         0x00, 0x00, 0x00, 0x10,
    //         0x00, 0x00, 0x00, 0x01,
    //         0x00, 0x00, 0x00, 0x00,
    //         0x00, 0x00, 0x00, 0x03,
    //         0x00, 0x00, 0x00, 0x04,
    //     ];

    //     let ConnectResponse { client_node, server_node } = ConnectResponse::deserialize(
    //         &mut std::io::Cursor::new(&input[..])
    //     ).await.unwrap();

    //     assert_eq!(client_node, 3);
    //     assert_eq!(server_node, 4);
    // }
}
