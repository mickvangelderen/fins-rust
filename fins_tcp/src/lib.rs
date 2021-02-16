use std::convert::{TryInto};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::io::BufStream;
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
        tokio::io::AsyncWriteExt::flush(&mut stream).await?;

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
    pub async fn serialize<W: tokio::io::AsyncWrite + Unpin>(&self, writer: &mut W) -> std::io::Result<()> {
        tokio::io::AsyncWriteExt::write_all(writer, self.raw().as_bytes()).await?;
        Ok(())
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

#[derive(Default, Debug)]
#[repr(C)]
struct RawConnectRequest {
    fins: [u8; 4],
    length: u32be,
    command: u32be,
    error_code: u32be,
    client_node: u32be,
}

impl_as_bytes!(RawConnectRequest);

#[derive(Debug)]
pub enum Frame {
    ConnectRequest(ConnectRequest),
    ConnectResponse(ConnectResponse),
}

#[derive(Debug)]
pub struct ConnectResponse {
    client_node: u8,
    server_node: u8,
}

impl ConnectResponse {
    pub async fn deserialize<R: tokio::io::AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<Self> {
        read_fins(reader).await?;

        let length = read_u32be(reader).await?;
        assert_eq!(16, length);

        let command = read_u32be(reader).await?;
        assert_eq!(1, command);

        let error_code = read_u32be(reader).await?;
        assert_eq!(0, error_code);

        let client_node = read_u32be(reader).await?;
        let server_node = read_u32be(reader).await?;

        Ok(ConnectResponse {
            client_node: client_node.try_into().unwrap(),
            server_node: server_node.try_into().unwrap(),
        })
    }
}

async fn read_4<R: tokio::io::AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<[u8; 4]> {
    let mut bytes = [0; 4];
    tokio::io::AsyncReadExt::read_exact(reader, &mut bytes).await?;
    Ok(bytes)
}

async fn read_fins<R: tokio::io::AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<()> {
    let bytes = read_4(reader).await?;
    assert_eq!(*b"FINS", bytes);
    Ok(())
}

async fn read_u32be<R: tokio::io::AsyncRead + Unpin>(reader: &mut R) -> std::io::Result<u32> {
    read_4(reader).await.map(u32::from_be_bytes)
}
