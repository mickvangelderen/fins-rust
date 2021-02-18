use std::convert::TryInto;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufStream};
use fins_util::*;

#[derive(Debug)]
pub enum Error {
    Invalid,
    UnknownCommand(RawCommandCode),
    UnexpectedFrame(Box<Frame>),
    ErrorCode { command: CommandCode, error_code: u32 },
    Io(std::io::Error)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Invalid => "Received invalid FINS/TCP frame.".fmt(f),
            Self::UnknownCommand(command) => write!(f, "Received FINS/TCP frame with unknown command {:?}.", command),
            Self::UnexpectedFrame(frame) => write!(f, "Received unexpected FINS/TCP frame {:?}.", frame),
            &Self::ErrorCode { command, error_code } => write!(f, "Received FINS/TCP error frame with command {:?} and error code {}.", command, error_code),
            Self::Io(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        match error {
            e if e.kind() == std::io::ErrorKind::UnexpectedEof => Self::Invalid,
            e => Self::Io(e)
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

pub struct FinsTcpStream {
    stream: BufStream<TcpStream>,
    pub client_node: u8,
    pub server_node: u8,
}

impl FinsTcpStream {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;

        stream.set_nodelay(true)?;

        let mut stream = BufStream::new(stream);

        ClientAddressFrame { client_node : 0 }.write_to(&mut stream).await?;
        stream.flush().await?;

        let ServerAddressFrame { client_node, server_node } = match Frame::read_from(&mut stream).await? {
            Frame::ServerAddress(frame) => frame,
            other => return Err(Error::UnexpectedFrame(Box::new(other))),
        };

        Ok(Self {
            stream,
            client_node,
            server_node,
        })
    }

    pub async fn write_frame(&mut self, frame: fins::Frame) -> Result<()> {
        write_raw!(&mut self.stream, Header {
            length: 8 + frame.byte_len(),
            command: CommandCode::Fins,
            error_code: 0,
        }.to_raw());
        frame.write_to(&mut self.stream).await?;
        self.stream.flush().await?;
        Ok(())
    }

    pub async fn read_frame(&mut self) -> Result<fins::Frame> {
        match Frame::read_from(&mut self.stream).await? {
            Frame::Fins(frame) => Ok(frame),
            other => Err(Error::UnexpectedFrame(Box::new(other))),
        }
    }

    pub fn stream(&self) -> &TcpStream {
        self.stream.get_ref()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ClientAddressFrame {
    pub client_node: u8
}

const FINS: [u8; 4] = *b"FINS";

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct RawCommandCode(u32be);

impl RawCommandCode {
    pub const CLIENT_ADDRESS: Self = Self(u32be::from_u32(0));
    pub const SERVER_ADDRESS: Self = Self(u32be::from_u32(1));
    pub const FINS: Self = Self(u32be::from_u32(2));
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CommandCode {
    ClientAddress,
    ServerAddress,
    Fins
}

impl CommandCode {
    pub const fn to_raw(&self) -> RawCommandCode {
        match self {
            CommandCode::ClientAddress => RawCommandCode::CLIENT_ADDRESS,
            CommandCode::ServerAddress => RawCommandCode::SERVER_ADDRESS,
            CommandCode::Fins => RawCommandCode::FINS,
        }
    }

    pub const fn from_raw(val: RawCommandCode) -> Result<Self> {
        match val {
            RawCommandCode::CLIENT_ADDRESS => Ok(Self::ClientAddress),
            RawCommandCode::SERVER_ADDRESS => Ok(Self::ServerAddress),
            RawCommandCode::FINS => Ok(Self::Fins),
            other => Err(Error::UnknownCommand(other)),
        }
    }
}

const CLIENT_ADDRESS_LENGTH: u32 = 12;
const SERVER_ADDRESS_LENGTH: u32 = 16;

impl ClientAddressFrame {
    pub async fn write_to<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
        write_raw!(writer, self.to_raw());
        Ok(())
    }

    pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let header = Header::from_raw(read_raw!(reader, RawHeader))?;
        if header.length != CLIENT_ADDRESS_LENGTH { return Err(Error::Invalid) }
        if header.command != CommandCode::ClientAddress { return Err(Error::Invalid) }
        if header.error_code != 0 { return Err(Error::Invalid) }
        let body = read_raw!(reader, RawClientAddressBody);
        Ok(Self::from_raw_body(body))
    }
    
    fn to_raw(&self) -> RawClientAddressFrame {
        RawClientAddressFrame {
            header: Header {
                length: CLIENT_ADDRESS_LENGTH,
                command: CommandCode::ClientAddress,
                error_code: 0
            }.to_raw(),
            body: RawClientAddressBody {
                client_node: u32be::from(self.client_node as u32)
            },
        }
    }

    fn from_raw_body(body: RawClientAddressBody) -> Self {
        Self {
            client_node: u32::from(body.client_node).try_into().unwrap(),
        }
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

#[derive(Debug, Eq, PartialEq)]
pub struct ServerAddressFrame {
    client_node: u8,
    server_node: u8,
}

impl ServerAddressFrame {
    pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let Header { length, command, error_code } = Header::from_raw(read_raw!(reader, RawHeader))?;
        if length != SERVER_ADDRESS_LENGTH { return Err(Error::Invalid) }
        if command != CommandCode::ServerAddress { return Err(Error::Invalid) }
        if error_code != 0 { return Err(Error::ErrorCode { command, error_code }) }
        let body = read_raw!(reader, RawServerAddressBody);
        Ok(Self::from_raw_body(body))
    }

    fn from_raw_body(body: RawServerAddressBody) -> Self {
        Self {
            client_node: u32::from(body.client_node).try_into().unwrap(),
            server_node: u32::from(body.server_node).try_into().unwrap(),
        }
    }
}

#[derive(Default)]
#[repr(C, packed)]
pub struct RawServerAddressFrame {
    header: RawHeader,
    body: RawServerAddressBody,
}

unsafe_impl_raw!(RawServerAddressFrame);


#[derive(Default)]
#[repr(C, packed)]
struct RawServerAddressBody {
    client_node: u32be,
    server_node: u32be,
}

unsafe_impl_raw!(RawServerAddressBody);

#[derive(Debug)]
pub enum Frame {
    ClientAddress(ClientAddressFrame),
    ServerAddress(ServerAddressFrame),
    Fins(fins::Frame),
}

impl Frame {
    // async fn write_to<W: AsyncWrite + Unpin>(&self, writer: &mut W) -> Result<()> {
    //     match self {
    //         Frame::ClientAddress(frame) => frame.write_to(writer).await?,
    //         Frame::ServerAddress(frame) => frame.write_to(writer).await?,
    //         Frame::Fins(frame) => {
    //             RawHeader {
    //                 fins: FINS,
    //                 length: (8 + frame.byte_len()).into(),
    //                 command: 2.into(),
    //                 error_code: 0.into(),
    //             }.write_to(writer).await?;
    //             frame.write_to(writer).await?;
    //         }
    //     }
    //     Ok(())
    // }

    pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let Header { length, command, error_code } = Header::from_raw(read_raw!(reader, RawHeader))?;

        // FIXME: Figure out what is sent on error. We don't want to break the stream by not reading the entire frame!
        if error_code != 0 {
            return Err(Error::ErrorCode { command, error_code })
        }

        Ok(match command {
            CommandCode::ClientAddress => {
                if length != CLIENT_ADDRESS_LENGTH { return Err(Error::Invalid) };
                let body = read_raw!(reader, RawClientAddressBody);
                Frame::ClientAddress(ClientAddressFrame::from_raw_body(body))
            }
            CommandCode::ServerAddress => {
                if length != SERVER_ADDRESS_LENGTH { return Err(Error::Invalid) };
                let body = read_raw!(reader, RawServerAddressBody);
                Frame::ServerAddress(ServerAddressFrame::from_raw_body(body))
            }
            CommandCode::Fins => unimplemented!()
        })
    }
}

#[derive(Debug)]
struct Header {
    length: u32,
    command: CommandCode,
    error_code: u32,
}

impl Header {
    fn to_raw(&self) -> RawHeader {
        RawHeader {
            fins: FINS,
            length: self.length.into(),
            command: self.command.to_raw(),
            error_code: self.error_code.into(),
        }
    }

    fn from_raw(val: RawHeader) -> Result<Self> {
        if val.fins != FINS { return Err(Error::Invalid); }
        Ok(Self {
            length: val.length.to_u32(),
            command: CommandCode::from_raw(val.command)?,
            error_code: val.error_code.to_u32(),
        })
    }
}

#[derive(Debug, Default, Copy, Clone)]
#[repr(C, packed)]
struct RawHeader {
    fins: [u8; 4],
    length: u32be,
    command: RawCommandCode,
    error_code: u32be,
}

unsafe_impl_raw!(RawHeader);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn connect_request_serializes() {
        let input = ClientAddressFrame { client_node: 3 };

        let output = {
            let mut bytes = [0u8; std::mem::size_of::<RawClientAddressFrame>()];
            input.write_to(&mut std::io::Cursor::new(&mut bytes[..])).await.unwrap();
            bytes
        };

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

        let output = ServerAddressFrame::read_from(&mut std::io::Cursor::new(&input[..])).await.unwrap();

        assert_eq!(output, ServerAddressFrame {
            client_node: 3,
            server_node: 4
        });
    }
}
