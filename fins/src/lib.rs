#![macro_use]

mod error;
mod header;
mod information_control_field;
mod machine_address;
mod memory_address;
mod memory_area_code;
mod memory_area_read_request;
mod protocol_violation;

pub use error::*;
pub use header::*;
pub use information_control_field::*;
pub use machine_address::*;
pub use memory_address::*;
pub use memory_area_code::*;
pub use memory_area_read_request::*;
pub use protocol_violation::*;

use fins_util::*;

#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct RawRequestHeader {
    /// Main Request Code
    pub mrc: u8,

    /// Sub Request Code
    pub src: u8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RequestHeader {
    MemoryAreaRead
}

impl RequestHeader {
    pub fn to_raw(self) -> RawRequestHeader {
        match self {
            Self::MemoryAreaRead => RawRequestHeader {
                mrc: 0x01,
                src: 0x01,
            }
        }
    }
}

unsafe_impl_raw!(RawRequestHeader);

#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct RawResponseHeader {
    /// Main Request Code
    pub mrc: u8,

    /// Sub Request Code
    pub src: u8,

    /// Main Response Code
    pub mres: u8,

    /// Sub Response Code
    pub sres: u8,
}

unsafe_impl_raw!(RawResponseHeader);
