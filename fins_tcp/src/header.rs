use std::io::{Read, Write};

use crate::command_code::CommandCode;
use crate::{ProtocolViolation, RawCommandCode};
use fins_util::{u32be, unsafe_impl_raw, ReadExt, WriteExt};

const FINS: [u8; 4] = *b"FINS";

#[derive(Debug, Default, Copy, Clone)]
#[repr(C, packed)]
pub struct RawHeader {
    pub fins: [u8; 4],
    pub length: u32be,
    pub command: RawCommandCode,
    pub error_code: u32be,
}

unsafe_impl_raw!(RawHeader);

#[derive(Debug)]
pub struct Header {
    pub length: u32,
    pub command: CommandCode,
    pub error_code: u32,
}

impl Header {
    pub fn read_from<R: Read>(reader: &mut R) -> crate::Result<Self> {
        Ok(Self::from_raw(reader.read_raw::<RawHeader>()?)?)
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> crate::Result<()> {
        writer.write_raw(&self.to_raw())?;
        Ok(())
    }

    fn to_raw(&self) -> RawHeader {
        RawHeader {
            fins: FINS,
            length: self.length.into(),
            command: self.command.to_raw(),
            error_code: self.error_code.into(),
        }
    }

    fn from_raw(val: RawHeader) -> Result<Self, ProtocolViolation> {
        if val.fins != FINS {
            return Err(ProtocolViolation::IncorrectMagicString(val.fins));
        }
        Ok(Self {
            length: val.length.to_u32(),
            command: CommandCode::from_raw(val.command)?,
            error_code: val.error_code.to_u32(),
        })
    }
}
