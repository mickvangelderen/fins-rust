use tracing::info;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let peer_addr: SocketAddr = "10.202.8.211:9600".parse()?;

    info!("attempting to connect to {}", peer_addr);

    let mut conn = fins_tcp::Connection::connect(peer_addr).await?;

    info!("connection established with {}", conn.stream().peer_addr()?);

    conn.write_frame(fins_tcp::Frame {
        body: {
            let mut bytes = vec![0u8; 12];
            fins::RequestFrame {
                client_node: conn.client_node,
                server_node: conn.server_node,
                service_id: 0,
                mrc: 1,
                src: 1,
                body: vec![
                    0x82,
                    0x00,
                    0x64,
                    0x00,
                    0x00,
                    0x96,
                ],
            }.write_to(&mut std::io::Cursor::new(&mut bytes)).await.unwrap();
            bytes
        }
    }).await?;

    let response = conn.read_frame().await?;

    print_bytes(&response.body);

    Ok(())
}

pub fn print_bytes(bytes: &[u8]) {
    for (index, &byte) in bytes.iter().enumerate() {
        println!("0x{:04X}: 0x{:02X} == {:3} == {}", index, byte, byte, if byte.is_ascii_graphic() { byte as char } else { ' ' });
    }
}
