use tokio::{io::{AsyncRead, AsyncWrite}, net::{TcpStream}};
use tokio::io::{BufStream, AsyncWriteExt, AsyncReadExt};
use tracing::info;
use std::{convert::TryInto, net::SocketAddr};

mod fins;
mod util;
mod raw;

use raw::*;
use fins::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let peer_addr: SocketAddr = "10.202.8.211:9600".parse()?;

    info!("attempting to connect to {}", peer_addr);

    let mut stream = TcpStream::connect(peer_addr).await?;

    info!("connection established with {}", stream.peer_addr()?);

    stream.set_nodelay(true)?;

    let mut stream = BufStream::new(stream);

    let start_instant = std::time::Instant::now();

    let request = RawConnectRequest {
        fins: *b"FINS",
        length: u32be::from_ne(12),
        command: u32be::from_ne(0),
        error_code: u32be::from_ne(0),
        client_node: u32be::from_ne(0),
    };
    stream.write_all(request.bytes()).await?;
    stream.flush().await?;

    let mut response = RawConnectResponse::uninit();
    stream.read_exact(response.bytes_mut()).await?;
    assert_eq!(*b"FINS", response.fins);
    assert_eq!(u32be::from_ne(1), response.command);
    assert_eq!(u32be::from_ne(0), response.error_code);

    info!("received {:?} in {:?}", response, start_instant.elapsed());

    let client_node: u8 = response.client_node.to_ne().try_into().unwrap();
    let server_node: u8 = response.server_node.to_ne().try_into().unwrap();

    let request_frame_header = RawFrameHeader {
        fins: *b"FINS",
        length: u32be::from_ne(8 + 18),
        command: u32be::from_ne(2),
        error_code: u32be::from_ne(0),
    };

    let mut sid = 0;
    
    let fins_frame = RawHeader {
        icf: 0x80,
        rsv: 0,
        gct: 0x02,
        dna: 0x00,
        da1: server_node,
        da2: 0x00,
        sna: 0x00,
        sa1: client_node,
        sa2: 0x00,
        sid: { sid += 1; sid },
    };

    stream.write_all(request_frame_header.bytes()).await?;
    stream.write_all(fins_frame.bytes()).await?;
    stream.write_all(&[
        0x01, // MRC
        0x01, // SRC
        0x82,
        0x00,
        0x64,
        0x00,
        0x00,
        0x96,
    ]).await?;
    stream.flush().await?;

    loop {
        let mut buffer = vec![0; 4096];
        match stream.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => util::print_bytes(&buffer[..n]),
            Err(e) => {
                tracing::error!("Failed to read: {:?}", e);
                break;
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct FinsTcpHeader {
    length: u32,
    command: u32,
    error_code: u32,
    client_node_add: u32,
}

impl FinsTcpHeader {
    pub fn from_raw(raw: &RawFinsTcpHeader) -> Self {
        assert_eq!(*b"FINS", raw.fins);

        FinsTcpHeader {
            length: raw.length.into(),
            command: raw.command.into(),
            error_code: raw.error_code.into(),
            client_node_add: raw.client_node_add.into(),
        }
    }

    pub fn to_raw(&self) -> RawFinsTcpHeader {
        RawFinsTcpHeader {
            fins: *b"FINS",
            length: self.length.into(),
            command: self.command.into(),
            error_code: self.error_code.into(),
            client_node_add: self.client_node_add.into(),
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

unsafe_impl_raw!(RawConnectRequest);

#[derive(Default, Debug)]
#[repr(C)]
struct RawConnectResponse {
    fins: [u8; 4],
    length: u32be,
    command: u32be,
    error_code: u32be,
    client_node: u32be,
    server_node: u32be,
}

unsafe_impl_raw!(RawConnectResponse);

#[derive(Debug)]
#[repr(C)]
struct RawFrameHeader {
    fins: [u8; 4],
    length: u32be,
    command: u32be,
    error_code: u32be,
}

unsafe_impl_raw!(RawFrameHeader);

#[derive(Default, Debug)]
#[repr(C)]
struct RawFinsTcpHeader {
    fins: [u8; 4],
    length: u32be,
    command: u32be,
    error_code: u32be,
    client_node_add: u32be,
}

unsafe_impl_raw!(RawFinsTcpHeader);

impl RawFinsTcpHeader {
    pub async fn write_async<W: AsyncWrite + Unpin>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(self.bytes()).await
    }

    pub async fn read_async<R: AsyncRead + Unpin>(r: &mut R) -> std::io::Result<Self> {
        let mut raw = Self::uninit();
        r.read_exact(raw.bytes_mut()).await?;
        println!("{:?}", &raw);
        Ok(raw)
    }
}

#[derive(Debug)]
#[repr(C, align(1))]
struct RawHeader {
    /// Information Control Field
    icf: u8,

    /// Reserved
    rsv: u8,
    
    /// Permissible number of gateways
    gct: u8,

    /// Destination Network Address
    dna: u8,

    /// Destination Node Address
    da1: u8,

    /// Destination Unit Address
    da2: u8,

    /// Source Network Address
    sna: u8,

    /// Source Node Address
    sa1: u8,

    /// Source Unit Address
    sa2: u8,

    /// Service ID
    /// Set by process to identify which one it came from.
    sid: u8,
}

unsafe_impl_raw!(RawHeader);

#[derive(Debug)]
#[repr(C, align(1))]
struct RawCommand {
    mrc: u8,
    src: u8,
}

unsafe_impl_raw!(RawCommand);
