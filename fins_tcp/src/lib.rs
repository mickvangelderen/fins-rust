use std::convert::{TryInto};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufStream};
use fins_util::*;

// #[derive(Debug)]
// pub enum Error {
//     Incomplete,
//     Io(std::io::Error)
// }

// impl std::fmt::Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match self {
//             Self::Incomplete => "Incomplete frame".fmt(f),
//             Self::Io(e) => e.fmt(f),
//         }
//     }
// }

// impl std::error::Error for Error {}

// impl From<std::io::Error> for Error {
//     fn from(error: std::io::Error) -> Self {
//         match error {
//             e if e.kind() == std::io::ErrorKind::UnexpectedEof => Self::Incomplete,
//             e => Self::Io(e)
//         }
//     }
// }

// type Result<T> = std::result::Result<T, Error>;
pub struct Connection {
    stream: BufStream<TcpStream>,
    client_node: u8,
    server_node: u8,
}

impl Connection {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;

        stream.set_nodelay(true)?;

        let mut stream = BufStream::new(stream);

        ConnectRequest { client_node: 0 }.serialize(&mut stream).await?;
        stream.flush().await?;

        let ConnectResponse { client_node, server_node } = ConnectResponse::deserialize(&mut stream).await?;

        Ok(Self {
            stream,
            client_node,
            server_node,
        })
    }

    pub fn stream(&self) -> &TcpStream {
        self.stream.get_ref()
    }
}

#[derive(Debug)]
pub struct ConnectRequest {
    pub client_node: u8
}

const FINS: [u8; 4] = *b"FINS";

impl ConnectRequest {
    pub async fn serialize<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> std::io::Result<()> {
        self.raw().write(writer).await
    }

    fn raw(&self) -> RawConnectRequest {
        RawConnectRequest {
            fins: FINS,
            length: (std::mem::size_of::<RawConnectRequest>() as u32 - 8).into(),
            command: 0.into(),
            error_code: 0.into(),
            client_node: (self.client_node as u32).into()
        }
    }
}

#[repr(C)]
struct RawConnectRequest {
    fins: [u8; 4],
    length: u32be,
    command: u32be,
    error_code: u32be,
    client_node: u32be,
}

unsafe_impl_raw!(RawConnectRequest);

#[derive(Debug)]
pub struct ConnectResponse {
    client_node: u8,
    server_node: u8,
}

impl ConnectResponse {
    pub async fn deserialize<R: AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<Self> {
        RawConnectResponse::read(reader).await.map(Self::from_raw)
    }

    fn from_raw(response: RawConnectResponse) -> Self {
        assert_eq!(FINS, response.fins);
        assert_eq!(u32be::from_ne(16), response.length);
        assert_eq!(u32be::from_ne(1), response.command);
        assert_eq!(u32be::from_ne(0), response.error_code);

        ConnectResponse {
            client_node: u32::from(response.client_node).try_into().unwrap(),
            server_node: u32::from(response.server_node).try_into().unwrap(),
        }
    }
}

#[repr(C)]
pub struct RawConnectResponse {
    fins: [u8; 4],
    length: u32be,
    command: u32be,
    error_code: u32be,
    client_node: u32be,
    server_node: u32be,
}

unsafe_impl_raw!(RawConnectResponse);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_request_serializes() {
        let mut output = [0u8; 20];

        ConnectRequest { client_node: 3 }.serialize(
            &mut std::io::Cursor::new(&mut output[..])
        ).await.unwrap();

        assert_eq!(output, [
            0x46, 0x49, 0x4E, 0x53,
            0x00, 0x00, 0x00, 0x0C,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x03,
        ]);
    }

    #[tokio::test]
    async fn connect_response_deserializes() {
        let input = [
            0x46, 0x49, 0x4E, 0x53,
            0x00, 0x00, 0x00, 0x10,
            0x00, 0x00, 0x00, 0x01,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x03,
            0x00, 0x00, 0x00, 0x04,
        ];

        let ConnectResponse { client_node, server_node } = ConnectResponse::deserialize(
            &mut std::io::Cursor::new(&input[..])
        ).await.unwrap();

        assert_eq!(client_node, 3);
        assert_eq!(server_node, 4);
    }
}
