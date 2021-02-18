use crate::*;

#[derive(Debug)]
pub enum Error {
    InvalidMemoryAddressCode(RawMemoryAreaCode),
    InvalidInformationControlField(RawInformationControlField),
}


impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidMemoryAddressCode(val) => write!(f, "Invalid FINS memory area code: {:?}", val),
            Self::InvalidInformationControlField(val) => write!(f, "Invalid FINS information control field: {:?}", val),
        }
    }
}
