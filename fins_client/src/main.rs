use fins::{MemoryAddress, MemoryAreaCode, MemoryAreaReadRequest};
use fins_tcp::{
    read_memory_area_read_response, write_memory_area_read_request, ClientAddressFrame,
    MemoryAreaReadResponse, ServerAddressFrame,
};
use std::io::Cursor;
use std::{fmt::write, io::ErrorKind, net::SocketAddr};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "fins_client=info".to_string()),
        )
        .init();

    tokio::select! {
        result = run() => {
            result?
        }
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

    let stream = tokio::net::TcpStream::connect(peer_addr).await?;
    info!("connection established with {}", stream.peer_addr()?);
    let (mut reader, mut writer) = stream.into_split();

    let mut write_buffer = Vec::with_capacity(2048);

    // Exchange nodes
    let mut write_cursor = Cursor::new(&mut write_buffer);
    ClientAddressFrame { client_node: 0 }.write_to(&mut write_cursor)?;
    let written = write_cursor.position();
    drop(write_cursor);
    writer.write_all(&write_buffer[..written as usize]).await?;
    writer.flush().await?;

    let mut read_buffer = vec![0; 2048];
    let mut read_position: usize = 0;

    let ServerAddressFrame {
        client_node,
        server_node,
    } = read_server_address_frame_async(&mut reader, &mut read_buffer, &mut read_position).await?;

    info!("client node {}, server node {}", client_node, server_node);

    // Memory area read
    let memory_area_read_request = MemoryAreaReadRequest {
        client_node,
        server_node,
        address: MemoryAddress {
            area_code: MemoryAreaCode::D,
            offset: 1500,
            bits: 0,
        },
        count: 16,
    };

    let pipeline_count = 32;

    // Send a bunch of requests without reading replies
    for _ in 0..pipeline_count {
        let mut write_cursor = Cursor::new(&mut write_buffer);
        write_memory_area_read_request(&mut write_cursor, &memory_area_read_request).unwrap();
        let written = write_cursor.position();
        drop(write_cursor);
        writer.write_all(&write_buffer[..written as usize]).await?;
        writer.flush().await?;
    }

    for _ in 0..pipeline_count {
        let MemoryAreaReadResponse {
            src_addr,
            dst_addr,
            bytes,
        } = read_memory_area_read_response_async(&mut reader, &mut read_buffer, &mut read_position)
            .await?;
    
        assert_eq!(src_addr.node, server_node);
        assert_eq!(dst_addr.node, client_node);
    
        print_bytes(memory_area_read_request.address, &bytes);
    }
    
    Ok(())
}

pub async fn read_server_address_frame_async<R: AsyncRead + Unpin>(
    reader: &mut R,
    read_buffer: &mut [u8],
    read_position: &mut usize,
) -> Result<ServerAddressFrame, Box<dyn std::error::Error>> {
    loop {
        if *read_position == read_buffer.len() {
            panic!("Frame does not fit in read buffer!");
        }
        match reader.read(&mut read_buffer[*read_position..]).await {
            Ok(0) => {
                return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into());
            }
            Ok(n) => {
                *read_position += n;
                let mut cursor = Cursor::new(&read_buffer[0..*read_position]);
                match ServerAddressFrame::read_from(&mut cursor) {
                    Ok(frame) => {
                        let consumed_position = cursor.position() as usize;
                        drop(cursor);
                        read_buffer.copy_within(consumed_position..*read_position, 0);
                        *read_position = 0;
                        return Ok(frame);
                    }
                    Err(fins_tcp::Error::Io(err)) if err.kind() == ErrorKind::UnexpectedEof => {
                        // Continue reading.
                    }
                    Err(err) => return Err(err.into()),
                }
            }
            Err(err) => return Err(err.into()),
        }
    }
}

pub async fn read_memory_area_read_response_async<R: AsyncRead + Unpin>(
    reader: &mut R,
    read_buffer: &mut [u8],
    read_position: &mut usize,
) -> Result<MemoryAreaReadResponse, Box<dyn std::error::Error>> {
    loop {
        if *read_position == read_buffer.len() {
            panic!("Frame does not fit in read buffer!");
        }
        match reader.read(&mut read_buffer[*read_position..]).await {
            Ok(0) => {
                return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into());
            }
            Ok(n) => {
                *read_position += n;
                let mut cursor = Cursor::new(&read_buffer[0..*read_position]);
                match read_memory_area_read_response(&mut cursor) {
                    Ok(frame) => {
                        let consumed_position = cursor.position() as usize;
                        drop(cursor);
                        read_buffer.copy_within(consumed_position..*read_position, 0);
                        *read_position = 0;
                        return Ok(frame)
                    },
                    Err(fins_tcp::Error::Io(err)) if err.kind() == ErrorKind::UnexpectedEof => {
                        // Continue reading.
                    }
                    Err(err) => return Err(err.into()),
                }
            }
            Err(err) => return Err(err.into()),
        }
    }
}

// pub async fn read_print(
//     conn: &mut fins_tcp::FinsTcpStream,
//     offset: u16,
//     count: u16,
// ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
//     let mem_addr = MemoryAddress {
//         area_code: MemoryAreaCode::D,
//         offset,
//         bits: 0,
//     };
//     let bytes = conn.read(mem_addr, count).await?;
//     print_bytes(mem_addr, &bytes);
//     Ok(bytes)
// }

pub fn print_bytes(mem_addr: MemoryAddress, bytes: &[u8]) {
    for index in (0..bytes.len()).step_by(2) {
        println!(
            "{0:>6}: 0x{1:02X} 0x{2:02X} | 0b{1:08b} 0b{2:08b} | {1:3} {2:3} | {3} {4} | {5} ",
            format!(
                "{:?}",
                MemoryAddress {
                    area_code: mem_addr.area_code,
                    offset: mem_addr.offset + index as u16,
                    bits: 0
                }
            ),
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
            u16::from_be_bytes([bytes[index], bytes[index + 1]])
        );
    }
}
