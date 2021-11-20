use crate::{CommandCode, RawCommandCode};

#[derive(Debug)]
#[non_exhaustive]
pub enum ProtocolViolation {
    IncorrectMagicString([u8; 4]),
    UnknownCommand(RawCommandCode),
    UnexpectedHeaderLength {
        actual: u32,
        expected: u32,
    },
    UnexpectedError(u32),
    UnexpectedCommand {
        actual: CommandCode,
        expected: CommandCode,
    },
    Fins(fins::ProtocolViolation),
}

impl From<fins::ProtocolViolation> for ProtocolViolation {
    fn from(error: fins::ProtocolViolation) -> Self {
        Self::Fins(error)
    }
}

impl std::fmt::Display for ProtocolViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::IncorrectMagicString(b) => write!(
                f,
                "Received incorrect magic string 0x{:02X}{:02X}{:02X}{:02X} but expected 0x46494E53 (\"FINS\")!",
                b[0], b[1], b[2], b[3]
            ),
            Self::UnknownCommand(command) => write!(
                f,
                "Received FINS/TCP frame with unknown command {}!",
                command.to_u32()
            ),
            Self::UnexpectedHeaderLength { .. } => todo!(),
            Self::UnexpectedError(_) => todo!(),
            Self::UnexpectedCommand { .. } => todo!(),
            Self::Fins(e) => e.fmt(f),
        }
    }
}

pub(crate) fn assert_header_length(actual: u32, expected: u32) -> Result<(), ProtocolViolation> {
    if actual == expected {
        Ok(())
    } else {
        Err(ProtocolViolation::UnexpectedHeaderLength { actual, expected })
    }
}

pub(crate) fn assert_command(
    actual: CommandCode,
    expected: CommandCode,
) -> Result<(), ProtocolViolation> {
    if actual == expected {
        Ok(())
    } else {
        Err(ProtocolViolation::UnexpectedCommand { actual, expected })
    }
}

pub(crate) fn assert_no_error(actual: u32) -> Result<(), ProtocolViolation> {
    if actual == 0 {
        Ok(())
    } else {
        Err(ProtocolViolation::UnexpectedError(actual))
    }
}

impl std::error::Error for ProtocolViolation {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_works() {
        assert_eq!(
            format!(
                "{}",
                ProtocolViolation::IncorrectMagicString([0x01, 0x52, 0x32, 0x99])
            ),
            "Received incorrect magic string 0x01523299 but expected 0x46494E53 (\"FINS\")!"
        );
    }
}
