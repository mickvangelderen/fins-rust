pub fn print_bytes(bytes: &[u8]) {
    for (index, &byte) in bytes.iter().enumerate() {
        println!("0x{:04X}: 0x{:02X} == {:3} == {}", index, byte, byte, if byte.is_ascii_graphic() { byte as char } else { ' ' });
    }
}
