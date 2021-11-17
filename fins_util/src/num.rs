macro_rules! impl_ints {
    ($BE:ident, $LE:ident, $N:ident, $B:ty, $to_ne:ident, $from_ne:ident) => {
        impl_ints!(@ $BE, $N, $B, $to_ne, $from_ne, to_be, from_be);
        impl_ints!(@ $LE, $N, $B, $to_ne, $from_ne, to_le, from_le);
    };
    (@ $TE:ident, $N:ident, $B:ty, $to_ne:ident, $from_ne:ident, $to_te:ident, $from_te:ident) => {
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone, Eq, PartialEq, Default)]
        #[repr(transparent)]
        pub struct $TE($N);

        impl $TE {
            #[inline(always)]
            pub const fn from_bytes(val: $B) -> Self {
                $TE($N::from_ne_bytes(val))
            }

            #[inline(always)]
            pub const fn to_bytes(self) -> $B {
                self.0.to_ne_bytes()
            }

            #[inline(always)]
            pub const fn $from_ne(val: $N) -> Self {
                Self(val.$to_te())
            }

            #[inline(always)]
            pub const fn $to_ne(self) -> $N {
                $N::$from_te(self.0)
            }
        }

        impl From<$N> for $TE {
            #[inline(always)]
            fn from(val: $N) -> Self {
                $TE::$from_ne(val)
            }
        }

        impl From<$TE> for $N {
            #[inline(always)]
            fn from(val: $TE) -> Self {
                $TE::$to_ne(val)
            }
        }

        impl std::fmt::Debug for $TE {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, concat!(stringify!($TE), "({})"), self.$to_ne())
            }
        }
    };
}

impl_ints!(u16be, u16le, u16, [u8; 2], to_u16, from_u16);
impl_ints!(i16be, i16le, i16, [u8; 2], to_i16, from_i16);
impl_ints!(u32be, u32le, u32, [u8; 4], to_u32, from_u32);
impl_ints!(i32be, i32le, i32, [u8; 4], to_i32, from_i32);
impl_ints!(u64be, u64le, u64, [u8; 8], to_u64, from_u64);
impl_ints!(i64be, i64le, i64, [u8; 8], to_i64, from_i64);
impl_ints!(u128be, u128le, u128, [u8; 16], to_u128, from_u128);
impl_ints!(i128be, i128le, i128, [u8; 16], to_i128, from_i128);

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    macro_rules! offset_of {
        ($Struct:path, $field:ident) => {{
            unsafe {
                let u = ::std::mem::MaybeUninit::<$Struct>::uninit();

                // Use pattern-matching to avoid accidentally going through Deref.
                let &$Struct { $field: ref f, .. } = &*u.as_ptr();

                let o = (f as *const _ as usize).wrapping_sub(&u as *const _ as usize);

                // Triple check that we are within `u` still.
                debug_assert!((0..=::std::mem::size_of_val(&u)).contains(&o));

                o
            }
        }};
    }

    #[test]
    fn types_are_aligned_to_their_native_counterparts() {
        assert_eq!(2, align_of::<i16be>());
        assert_eq!(2, align_of::<i16le>());
        assert_eq!(4, align_of::<i32be>());
        assert_eq!(4, align_of::<i32le>());
        assert_eq!(8, align_of::<i64be>());
        assert_eq!(8, align_of::<i64le>());
        assert_eq!(2, align_of::<u16be>());
        assert_eq!(2, align_of::<u16le>());
        assert_eq!(4, align_of::<u32be>());
        assert_eq!(4, align_of::<u32le>());
        assert_eq!(8, align_of::<u64be>());
        assert_eq!(8, align_of::<u64le>());
        assert_eq!(16, align_of::<u128be>());
        assert_eq!(16, align_of::<u128le>());
    }

    #[repr(C, packed)]
    struct RawThing {
        a: u8,
        b: u16be,
        c: u64be,
    }

    #[test]
    fn struct_is_byte_aligned() {
        assert_eq!(1, align_of::<RawThing>());
        assert_eq!(0, offset_of!(RawThing, a));
        assert_eq!(1, offset_of!(RawThing, b));
        assert_eq!(3, offset_of!(RawThing, c));
        assert_eq!(11, size_of::<RawThing>())
    }

    #[test]
    fn conversions_make_sense() {
        assert_eq!(
            i32be::from(0x11223344).to_bytes(),
            0x11223344i32.to_be_bytes()
        );
        assert_eq!(
            i32le::from(0x11223344).to_bytes(),
            0x11223344i32.to_le_bytes()
        );
    }
}
