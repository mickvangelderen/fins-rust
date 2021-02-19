#![macro_use]

mod error;
mod header;
mod information_control_field;
mod machine_address;
mod memory_address;
mod memory_area_code;

pub use error::*;
pub use header::*;
pub use information_control_field::*;
pub use machine_address::*;
pub use memory_address::*;
pub use memory_area_code::*;

use fins_util::*;

type Result<T> = std::result::Result<T, Error>;

#[macro_export]
macro_rules! trye {
    ($e:expr) => {
        match $e {
            Ok(val) => val,
            Err(err) => return Err(err),
        }
    };
}

#[derive(Debug, Default)]
#[repr(C, packed)]
pub struct RawRequestHeader {
    /// Main Request Code
    pub mrc: u8,

    /// Sub Request Code
    pub src: u8,
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

    /// Sub Repsonse Code
    pub sres: u8,
}

unsafe_impl_raw!(RawResponseHeader);
