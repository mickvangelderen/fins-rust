use crate::ProtocolViolation;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    ProtocolViolation(ProtocolViolation),
    Io(std::io::Error),
}

impl From<ProtocolViolation> for Error {
    fn from(error: ProtocolViolation) -> Self {
        Self::ProtocolViolation(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ProtocolViolation(e) => e.fmt(f),
            Self::Io(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
