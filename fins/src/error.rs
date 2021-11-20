use crate::*;

#[derive(Debug)]
pub enum Error {
    ProtocolViolation(ProtocolViolation),
    Io(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ProtocolViolation(e) => e.fmt(f),
            Self::Io(e) => e.fmt(f),
        }
    }
}

impl From<ProtocolViolation> for Error {
    fn from(e: ProtocolViolation) -> Self {
        Self::ProtocolViolation(e)
    }
}

impl From<::std::io::Error> for Error {
    fn from(e: ::std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl std::error::Error for Error {}

pub type Result<T, E = Error> = ::std::result::Result<T, E>;
