mod information_control_field;
mod memory_area_code;
mod memory_address;
mod machine_address;
mod header;
mod error;

pub use information_control_field::*;
pub use memory_area_code::*;
pub use memory_address::*;
pub use machine_address::*;
pub use header::*;
pub use error::*;

use fins_util::*;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Default)]
#[repr(C, packed)]
struct RawRequestHeader {
    /// Main Request Code
    pub mrc: u8,

    /// Sub Request Code
    pub src: u8,
}

unsafe_impl_raw!(RawRequestHeader);

#[derive(Debug, Default)]
#[repr(C, packed)]
struct RawResponseHeader {
    /// Main Request Code
    pub mrc: u8,

    /// Sub Request Code
    pub src: u8,

    /// Main Response Code
    pub mres: u8,

    /// Sub Repsonse Code
    pub sres: u8,
}

unsafe_impl_raw!(RawResponseHeader);
