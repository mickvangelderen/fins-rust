macro_rules! impl_ints {
    ($BE:ident, $LE:ident, $N:ident, $B:ty) => {
        impl_ints!(@ $BE, $N, $B, to_be_bytes, from_be_bytes);
        impl_ints!(@ $LE, $N, $B, to_le_bytes, from_le_bytes);
    };
    (@ $TE:ident, $N:ident, $B:ty, $into:ident, $from:ident) => {
        #[allow(non_camel_case_types)]
        #[derive(Copy, Clone, Eq, PartialEq, Default)]
        #[repr(transparent)]
        pub struct $TE($B);

        impl $TE {
            #[inline(always)]
            pub const fn from_bytes(val: $B) -> Self {
                $TE(val)
            }

            #[inline(always)]
            pub const fn to_bytes(self) -> $B {
                self.0
            }

            #[inline(always)]
            pub const fn from_ne(val: $N) -> Self {
                $TE($N::$into(val))
            }

            #[inline(always)]
            pub const fn to_ne(self) -> $N {
                $N::$from(self.0)
            }
        }

        impl From<$N> for $TE {
            #[inline(always)]
            fn from(val: $N) -> Self {
                $TE::from_ne(val)
            }
        }

        impl From<$TE> for $N {
            #[inline(always)]
            fn from(val: $TE) -> Self {
                $TE::to_ne(val)
            }
        }

        impl std::fmt::Debug for $TE {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, concat!(stringify!($TE), "({})"), self.to_ne())
            }
        }
    };
}

impl_ints!(u16be, u16le, u16, [u8; 2]);
impl_ints!(i16be, i16le, i16, [u8; 2]);
impl_ints!(u32be, u32le, u32, [u8; 4]);
impl_ints!(i32be, i32le, i32, [u8; 4]);
impl_ints!(u64be, u64le, u64, [u8; 8]);
impl_ints!(i64be, i64le, i64, [u8; 8]);

#[macro_export]
macro_rules! unsafe_impl_raw {
    ($T:ty) => {
        impl $T {
            #[inline(always)]
            fn uninit() -> Self {
                unsafe { std::mem::MaybeUninit::uninit().assume_init() }
            }
        
            #[inline(always)]
            fn bytes(&self) -> &[u8; std::mem::size_of::<Self>()] {
                unsafe { &*(self as *const Self as *const _) }
            }
        
            #[inline(always)]
            fn bytes_mut(&mut self) -> &mut [u8; std::mem::size_of::<Self>()] {
                unsafe { &mut*(self as *mut Self as *mut _) }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::align_of;

    #[test]
    fn types_are_byte_aligned() {
        assert_eq!(1, align_of::<i16be>());
        assert_eq!(1, align_of::<i16le>());
        assert_eq!(1, align_of::<i32be>());
        assert_eq!(1, align_of::<i32le>());
        assert_eq!(1, align_of::<i64be>());
        assert_eq!(1, align_of::<i64le>());
        assert_eq!(1, align_of::<u16be>());
        assert_eq!(1, align_of::<u16le>());
        assert_eq!(1, align_of::<u32be>());
        assert_eq!(1, align_of::<u32le>());
        assert_eq!(1, align_of::<u64be>());
        assert_eq!(1, align_of::<u64le>());
    }

    #[repr(C)]
    struct RawThing {
        a: u16be,
        b: u16be,
        c: u64be,
    }

    #[test]
    fn struct_is_byte_aligned() {
        assert_eq!(1, align_of::<RawThing>());
    }

    #[test]
    fn conversions_make_sense() {
        assert_eq!(i32be::from(0x11223344).to_bytes(), 0x11223344i32.to_be_bytes());
        assert_eq!(i32le::from(0x11223344).to_bytes(), 0x11223344i32.to_le_bytes());
    }
}
