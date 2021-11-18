use crate::ProtocolViolation;
use fins_util::u32be;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct RawCommandCode(u32be);

impl RawCommandCode {
    pub const CLIENT_ADDRESS: Self = Self(u32be::from_u32(0));
    pub const SERVER_ADDRESS: Self = Self(u32be::from_u32(1));
    pub const FINS: Self = Self(u32be::from_u32(2));

    pub const fn to_u32(self) -> u32 {
        self.0.to_u32()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CommandCode {
    ClientAddress,
    ServerAddress,
    Fins,
}

impl CommandCode {
    pub const fn to_raw(self) -> RawCommandCode {
        match self {
            CommandCode::ClientAddress => RawCommandCode::CLIENT_ADDRESS,
            CommandCode::ServerAddress => RawCommandCode::SERVER_ADDRESS,
            CommandCode::Fins => RawCommandCode::FINS,
        }
    }

    pub const fn from_raw(val: RawCommandCode) -> Result<Self, ProtocolViolation> {
        match val {
            RawCommandCode::CLIENT_ADDRESS => Ok(Self::ClientAddress),
            RawCommandCode::SERVER_ADDRESS => Ok(Self::ServerAddress),
            RawCommandCode::FINS => Ok(Self::Fins),
            other => Err(ProtocolViolation::UnknownCommand(other)),
        }
    }
}

impl std::fmt::Display for CommandCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
