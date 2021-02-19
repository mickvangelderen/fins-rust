use fins::{MemoryAddress, MemoryAreaCode};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    tokio::select! {
        _ = run() => {}
        _ = tokio::signal::ctrl_c() => {}
    }

    Ok(())
}

/*
[14:43:55 INF] New CommandHub started, BrowserGuid: null
HoldToRun.Instruction.Assign.SequenceNumber (D1508): 7466
HoldToRun.Instruction.Assign.Instruction (D1501): 0
HoldToRun.Instruction.Assign.Watchdog (D1500): 0
HoldToRun.Instruction.Assign.ManualMode (D1503): 1
HoldToRun.Instruction.Assign.MachineMode (D1502): 1
HoldToRun.Instruction.Status.SequenceNumber (D1600): 7466
Alarms.Status.Alarm_SlowCommunication (D2420.00): False
Alarms.Status.Alarm_NoCommunication (D2420.01): True
LightGroup2.Status.LED_100_02 (D2420.02): False
LightGroup2.Status.LED_100_06 (D2420.03): False
LightGroup2.Status.LED_100_07 (D2420.04): False
*/

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let peer_addr: SocketAddr = "10.202.8.211:9600".parse()?;

    info!("attempting to connect to {}", peer_addr);

    let mut conn = fins_tcp::FinsTcpStream::connect(peer_addr).await?;

    info!("connection established with {}", conn.stream().peer_addr()?);

    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1000));

    loop {
        let _ = interval.tick().await;

        read_print(&mut conn, 1500 + 0, 5).await?;
        read_print(&mut conn, 1500 + 100, 1).await?;
        read_print(&mut conn, 1500 + 920, 1).await?;
    }
}

pub async fn read_print(conn: &mut fins_tcp::FinsTcpStream, offset: u16, count: u16) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mem_addr = MemoryAddress::new(MemoryAreaCode::D, offset, 0);
    let bytes = conn.read(mem_addr, count).await?;
    print_bytes(mem_addr, &bytes);
    Ok(bytes)
}

pub fn print_bytes(mem_addr: MemoryAddress, bytes: &[u8]) {
    for index in (0..bytes.len()).step_by(2) {
        println!(
            "{0:>6}: 0x{1:02X} 0x{2:02X} | 0b{1:08b} 0b{2:08b} | {1:3} {2:3} | {3} {4} | {5} ",
            format!("{:?}", MemoryAddress::new(
                mem_addr.area_code(),
                mem_addr.offset() + index as u16,
                0
            )),
            bytes[index],
            bytes[index + 1],
            if bytes[index].is_ascii_graphic() {
                bytes[index] as char
            } else {
                ' '
            },
            if bytes[index + 1].is_ascii_graphic() {
                bytes[index + 1] as char
            } else {
                ' '
            },
            u16::from_be_bytes([ bytes[index], bytes[index + 1] ])
        );
    }
}
