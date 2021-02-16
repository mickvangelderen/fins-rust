use tracing::info;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let peer_addr: SocketAddr = "10.202.8.211:9600".parse()?;

    info!("attempting to connect to {}", peer_addr);

    let conn = fins_tcp::Connection::connect(peer_addr).await?;

    info!("connection established with {}", conn.stream().peer_addr()?);

    Ok(())
}
