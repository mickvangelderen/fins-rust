use crate::*;

const fn test_bits_u8(v: u8, b: u8) -> bool {
    v & b == b
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InformationControlField {
    RequestWithResponse,
    RequestWithoutResponse,
    ResponseWithResponse,
    ResponseWithoutResponse,
}

impl InformationControlField {
    pub const fn is_request(&self) -> bool {
        matches!(
            self,
            Self::RequestWithResponse | Self::RequestWithoutResponse
        )
    }

    pub const fn requires_response(&self) -> bool {
        matches!(self, Self::RequestWithResponse | Self::ResponseWithResponse)
    }

    pub const fn serialize(&self) -> RawInformationControlField {
        let mut bits = 0b10000000;
        if !self.requires_response() {
            bits |= 1 << 0;
        }
        if !self.is_request() {
            bits |= 1 << 6;
        }
        RawInformationControlField(bits)
    }
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, packed)]
pub struct RawInformationControlField(u8);

impl RawInformationControlField {
    pub const fn deserialize(self) -> Result<InformationControlField> {
        let bits = self.0;
        if bits & 0b10111110 != 0b10000000 {
            return Err(Error::InvalidInformationControlField(self));
        }
        let requires_response = !test_bits_u8(bits, 1 << 0);
        let is_command = !test_bits_u8(bits, 1 << 6);
        Ok(match (is_command, requires_response) {
            (true, true) => InformationControlField::RequestWithResponse,
            (true, false) => InformationControlField::RequestWithoutResponse,
            (false, true) => InformationControlField::ResponseWithResponse,
            (false, false) => InformationControlField::ResponseWithoutResponse,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_bidir {
        ($unraw:expr, $raw:expr $(,)?) => {
            assert_eq!($unraw.serialize(), $raw);
            assert_eq!($unraw, $raw.deserialize().unwrap());
        };
    }

    #[test]
    fn information_control_field_works() {
        test_bidir!(
            InformationControlField::RequestWithResponse,
            RawInformationControlField(0b10000000),
        );
        test_bidir!(
            InformationControlField::RequestWithoutResponse,
            RawInformationControlField(0b10000001),
        );
        test_bidir!(
            InformationControlField::ResponseWithResponse,
            RawInformationControlField(0b11000000),
        );
        test_bidir!(
            InformationControlField::ResponseWithoutResponse,
            RawInformationControlField(0b11000001),
        );
    }
}
