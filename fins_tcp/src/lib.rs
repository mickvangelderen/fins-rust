use fins::InformationControlField;
use fins_util::*;
use std::convert::TryInto;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufStream};
use tokio::net::{TcpStream, ToSocketAddrs};

#[derive(Debug)]
pub enum Error {
    Invalid,
    UnknownCommand(RawCommandCode),
    ErrorCode {
        command: CommandCode,
        error_code: u32,
    },
    Fins(fins::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Invalid => "Received invalid FINS/TCP frame.".fmt(f),
            Self::UnknownCommand(command) => write!(
                f,
                "Received FINS/TCP frame with unknown command {:?}.",
                command
            ),
            &Self::ErrorCode {
                command,
                error_code,
            } => write!(
                f,
                "Received FINS/TCP error frame with command {:?} and error code {}.",
                command, error_code
            ),
            Self::Fins(e) => e.fmt(f),
            Self::Io(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        match error {
            e if e.kind() == std::io::ErrorKind::UnexpectedEof => Self::Invalid,
            e => Self::Io(e),
        }
    }
}

impl From<fins::Error> for Error {
    fn from(error: fins::Error) -> Self {
        Self::Fins(error)
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

        ClientAddressFrame { client_node: 0 }
            .write_to(&mut stream)
            .await?;
        stream.flush().await?;

        let ServerAddressFrame {
            client_node,
            server_node,
        } = ServerAddressFrame::read_from(&mut stream).await?;

        Ok(Self {
            stream,
            client_node,
            server_node,
        })
    }

    pub async fn read(&mut self, address: fins::MemoryAddress, count: u16) -> Result<Vec<u8>> {
        {
            #[derive(Default)]
            #[repr(C, packed)]
            struct RawRequest {
                fins_tcp_header: RawHeader,
                fins_header: fins::RawHeader,
                fins_request: fins::RawRequestHeader,
                address: fins::RawMemoryAddress,
                count: u16be,
            }
            unsafe_impl_raw!(RawRequest);

            let request = RawRequest {
                fins_tcp_header: Header {
                    command: CommandCode::Fins,
                    length: std::mem::size_of::<RawRequest>() as u32 - 8,
                    error_code: 0,
                }
                .to_raw(),
                fins_header: fins::Header {
                    icf: fins::InformationControlField::RequestWithResponse,
                    gct: 0x02,
                    destination: fins::MachineAddress {
                        network: 0,
                        node: self.server_node,
                        unit: 0,
                    },
                    source: fins::MachineAddress {
                        network: 0,
                        node: self.client_node,
                        unit: 0,
                    },
                    sid: 0,
                }
                .serialize(),
                fins_request: fins::RawRequestHeader {
                    mrc: 0x01,
                    src: 0x01,
                },
                address: address.serialize(),
                count: u16be::from_u16(count),
            };

            write_raw!(&mut self.stream, request);
            self.stream.flush().await?;
        }

        {
            let Header {
                length,
                command,
                error_code,
            } = Header::from_raw(read_raw!(&mut self.stream, RawHeader))?;

            assert_eq!(command, CommandCode::Fins);
            assert_eq!(0, error_code);

            let fins::Header {
                icf,
                gct: _,
                destination,
                source,
                sid,
            } = read_raw!(&mut self.stream, fins::RawHeader).deserialize()?;

            assert_eq!(icf, InformationControlField::ResponseWithResponse);
            assert_eq!(
                destination,
                fins::MachineAddress {
                    network: 0,
                    node: self.client_node,
                    unit: 0
                }
            );
            assert_eq!(
                source,
                fins::MachineAddress {
                    network: 0,
                    node: self.server_node,
                    unit: 0
                }
            );
            assert_eq!(sid, 0); // whatever?

            let fins::RawResponseHeader {
                mrc,
                src,
                mres,
                sres,
            } = read_raw!(&mut self.stream, fins::RawResponseHeader);

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
            self.stream.read_exact(&mut bytes[..]).await?;

            Ok(bytes)
        }
    }

    pub fn stream(&self) -> &TcpStream {
        self.stream.get_ref()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct ClientAddressFrame {
    pub client_node: u8,
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
    Fins,
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
        if header.length != CLIENT_ADDRESS_LENGTH {
            return Err(Error::Invalid);
        }
        if header.command != CommandCode::ClientAddress {
            return Err(Error::Invalid);
        }
        if header.error_code != 0 {
            return Err(Error::Invalid);
        }
        let body = read_raw!(reader, RawClientAddressBody);
        Ok(Self::from_raw_body(body))
    }

    fn to_raw(&self) -> RawClientAddressFrame {
        RawClientAddressFrame {
            header: Header {
                length: CLIENT_ADDRESS_LENGTH,
                command: CommandCode::ClientAddress,
                error_code: 0,
            }
            .to_raw(),
            body: RawClientAddressBody {
                client_node: u32be::from(self.client_node as u32),
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
        let Header {
            length,
            command,
            error_code,
        } = Header::from_raw(read_raw!(reader, RawHeader))?;
        if length != SERVER_ADDRESS_LENGTH {
            return Err(Error::Invalid);
        }
        if command != CommandCode::ServerAddress {
            return Err(Error::Invalid);
        }
        if error_code != 0 {
            return Err(Error::ErrorCode {
                command,
                error_code,
            });
        }
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

impl FinsFrame {
    pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let fins::Header {
            icf,
            gct,
            destination,
            source,
            sid,
        } = read_raw!(reader, fins::RawHeader).deserialize()?;

        unimplemented!()

        // if icf.is_request() {
        //     let fins::RawRequestHeader { mrc, src } = read_raw!(reader, fins::RawRequestHeader);
        //     unimplemented!()
        // } else {
        //     let fins::RawResponseHeader { mrc, src, mres, sres } = read_raw!(reader, fins::RawResponseHeader);
        //     assert_eq!(mrc, 1);
        //     assert_eq!(src, 1);
        //     assert_eq!(mres, 0);
        //     assert_eq!(sres, 0);
        //     Ok(FinsFrame::Response(
        //         FinsResponseFrame::Read()
        //     ))
        // }
    }
}

#[derive(Debug)]
pub enum FinsTcpFrame {
    ClientAddress(ClientAddressFrame),
    ServerAddress(ServerAddressFrame),
    Fins(FinsFrame),
}

impl FinsTcpFrame {
    pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self> {
        let Header {
            length,
            command,
            error_code,
        } = Header::from_raw(read_raw!(reader, RawHeader))?;

        // FIXME: Figure out what is sent on error. We don't want to break the stream by not reading the entire frame!
        if error_code != 0 {
            return Err(Error::ErrorCode {
                command,
                error_code,
            });
        }

        Ok(match command {
            CommandCode::ClientAddress => {
                if length != CLIENT_ADDRESS_LENGTH {
                    return Err(Error::Invalid);
                };
                let body = read_raw!(reader, RawClientAddressBody);
                FinsTcpFrame::ClientAddress(ClientAddressFrame::from_raw_body(body))
            }
            CommandCode::ServerAddress => {
                if length != SERVER_ADDRESS_LENGTH {
                    return Err(Error::Invalid);
                };
                let body = read_raw!(reader, RawServerAddressBody);
                FinsTcpFrame::ServerAddress(ServerAddressFrame::from_raw_body(body))
            }
            CommandCode::Fins => unimplemented!(),
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
        if val.fins != FINS {
            return Err(Error::Invalid);
        }
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
            input
                .write_to(&mut std::io::Cursor::new(&mut bytes[..]))
                .await
                .unwrap();
            bytes
        };

        assert_eq!(
            output,
            [
                0x46, 0x49, 0x4E, 0x53, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x03,
            ]
        );
    }

    #[tokio::test]
    async fn connect_response_deserializes() {
        let input = [
            0x46, 0x49, 0x4E, 0x53, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04,
        ];

        let output = ServerAddressFrame::read_from(&mut std::io::Cursor::new(&input[..]))
            .await
            .unwrap();

        assert_eq!(
            output,
            ServerAddressFrame {
                client_node: 3,
                server_node: 4
            }
        );
    }
}
