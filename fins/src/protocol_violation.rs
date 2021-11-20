use crate::*;

#[derive(Debug)]
pub enum ProtocolViolation {
    InvalidMemoryAreaCode(RawMemoryAreaCode),
    InvalidInformationControlField(RawInformationControlField),
}

impl std::fmt::Display for ProtocolViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::InvalidMemoryAreaCode(val) => {
                write!(f, "Invalid FINS memory area code: {:?}", val)
            }
            Self::InvalidInformationControlField(val) => {
                write!(f, "Invalid FINS information control field: {:?}", val)
            }
        }
    }
}

impl std::error::Error for ProtocolViolation {}
