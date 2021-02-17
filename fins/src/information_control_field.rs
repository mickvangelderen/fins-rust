use fins_util::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InformationControlField {
    RequestWithResponse,
    RequestWithoutResponse,
    ResponseWithResponse,
    ResponseWithoutResponse,
}

impl InformationControlField {
    fn is_request(&self) -> bool {
        matches!(
            self,
            Self::RequestWithResponse | Self::RequestWithoutResponse
        )
    }

    fn requires_response(&self) -> bool {
        matches!(self, Self::RequestWithResponse | Self::ResponseWithResponse)
    }
}

fn test_bits_u8(v: u8, b: u8) -> bool {
    v & b == b
}

impl From<RawInformationControlField> for InformationControlField {
    fn from(raw: RawInformationControlField) -> Self {
        let bits = raw.0;
        debug_assert_eq!(0b10000000, bits & 0b10111110);
        let requires_response = !test_bits_u8(bits, 1 << 0);
        let is_command = !test_bits_u8(bits, 1 << 6);
        match (is_command, requires_response) {
            (true, true) => Self::RequestWithResponse,
            (true, false) => Self::RequestWithoutResponse,
            (false, true) => Self::ResponseWithResponse,
            (false, false) => Self::ResponseWithoutResponse,
        }
    }
}

impl From<InformationControlField> for RawInformationControlField {
    fn from(val: InformationControlField) -> Self {
        let mut bits = 0b10000000;
        if !val.requires_response() {
            bits |= 1 << 0;
        }
        if !val.is_request() {
            bits |= 1 << 6;
        }
        Self(bits)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct RawInformationControlField(u8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn information_control_field_works() {
        test_bidir_from(
            InformationControlField::RequestWithResponse,
            RawInformationControlField(0b10000000),
        );
        test_bidir_from(
            InformationControlField::RequestWithoutResponse,
            RawInformationControlField(0b10000001),
        );
        test_bidir_from(
            InformationControlField::ResponseWithResponse,
            RawInformationControlField(0b11000000),
        );
        test_bidir_from(
            InformationControlField::ResponseWithoutResponse,
            RawInformationControlField(0b11000001),
        );
    }

    fn test_bidir_from<A, B>(a: A, b: B)
    where
        A: PartialEq<A> + From<B> + Copy + std::fmt::Debug,
        B: PartialEq<B> + From<A> + Copy + std::fmt::Debug,
    {
        assert_eq!(a, A::from(b));
        assert_eq!(b, B::from(a));
    }
}
