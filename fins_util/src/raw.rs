use std::{
    io::{Read, Write},
    mem::MaybeUninit,
};

pub unsafe trait Raw: Sized {
    fn as_bytes(&self) -> &[u8];

    fn as_bytes_mut(&mut self) -> &mut [u8];
}

pub trait WriteExt {
    fn write_raw<T: Raw>(&mut self, raw: &T) -> std::io::Result<()>;
}

impl<W: Write> WriteExt for W {
    fn write_raw<T: Raw>(&mut self, raw: &T) -> std::io::Result<()> {
        self.write_all(raw.as_bytes())
    }
}

pub trait ReadExt {
    fn read_raw<T: Raw>(&mut self) -> std::io::Result<T>;
}

impl<R: Read> ReadExt for R {
    fn read_raw<T: Raw>(&mut self) -> std::io::Result<T> {
        unsafe {
            let mut value = MaybeUninit::<T>::uninit();
            let bytes = std::slice::from_raw_parts_mut(
                value.as_mut_ptr() as *mut u8,
                std::mem::size_of::<T>(),
            );
            self.read_exact(bytes)?;
            std::mem::forget(bytes);
            Ok(value.assume_init())
        }
    }
}

/// Promise that it is safe to transmute this type to and from its byte representation.
#[macro_export]
macro_rules! unsafe_impl_raw {
    ($T:ty) => {
        impl $T {
            #[inline]
            pub fn bytes(&self) -> &[u8; ::std::mem::size_of::<Self>()] {
                unsafe { &*(self as *const Self as *const _) }
            }

            #[inline]
            pub fn bytes_mut(&mut self) -> &mut [u8; ::std::mem::size_of::<Self>()] {
                unsafe { &mut *(self as *mut Self as *mut _) }
            }

            #[inline]
            pub fn ref_from_bytes_mut(
                bytes: &mut [u8],
            ) -> ::std::result::Result<(&mut Self, &mut [u8]), ()> {
                if ::std::mem::size_of::<Self>() <= bytes.len() {
                    let (a, b) = bytes.split_at_mut(::std::mem::size_of::<Self>());
                    return Ok((unsafe { &mut *(a.as_mut_ptr() as *mut Self) }, b));
                }
                Err(())
            }
        }

        unsafe impl $crate::Raw for $T {
            #[inline]
            fn as_bytes(&self) -> &[u8] {
                self.bytes()
            }

            #[inline]
            fn as_bytes_mut(&mut self) -> &mut [u8] {
                self.bytes_mut()
            }
        }
    };
}
