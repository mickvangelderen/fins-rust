use fins::{MemoryAddress, RequestFrame};
use tracing::info;
use std::net::SocketAddr;


struct MemoryRead {
    address: MemoryAddress,
    count: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let peer_addr: SocketAddr = "10.202.8.211:9600".parse()?;

    info!("attempting to connect to {}", peer_addr);

    let mut conn = fins_tcp::FinsTcpStream::connect(peer_addr).await?;

    info!("connection established with {}", conn.stream().peer_addr()?);

    conn.write_frame(fins::Frame::Request(fins::RequestFrame {
        client_node: conn.client_node,
        server_node: conn.server_node,
        service_id: 0,
        mrc: 1,
        src: 1,
        body: vec![
            0x82, // memory area code
            0x00, // memory address
            0x0A,
            0x00,
            0x00,
            0x96,
        ],
    })).await?;

    let response = match conn.read_frame().await? {
        fins::Frame::Request(request) => panic!("Unexpected request"),
        fins::Frame::Response(response) => response
    };

    print_bytes(&response.body);

    Ok(())
}

pub fn print_bytes(bytes: &[u8]) {
    for (index, &byte) in bytes.iter().enumerate() {
        println!("0x{:04X}: 0x{:02X} == {:3} == {}", index, byte, byte, if byte.is_ascii_graphic() { byte as char } else { ' ' });
    }
}
