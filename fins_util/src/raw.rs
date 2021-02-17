#[macro_export]
macro_rules! impl_as_bytes {
    ($T:ty) => {
        impl $T {
            #[inline(always)]
            fn as_bytes(&self) -> &[u8; std::mem::size_of::<Self>()] {
                unsafe { &*(self as *const Self as *const _) }
            }
        }
    }
}

#[macro_export]
macro_rules! unsafe_impl_raw {
    ($T:ty) => {
        impl $T {
            #[inline]
            async fn read_from<R: ::tokio::io::AsyncRead + ::std::marker::Unpin>(reader: &mut R) -> ::std::io::Result<Self> {
                unsafe {
                    // FIXME(mickvangelderen): I'm aware this is unsound but I refuse to initialize to zero.
                    // I will gamble on the fact that most reader implementations will write to all bytes
                    // or return an error. See https://www.ralfj.de/blog/2019/07/14/uninit.html
                    let mut value = ::std::mem::MaybeUninit::<Self>::uninit().assume_init();
                    ::tokio::io::AsyncReadExt::read_exact(reader, value.bytes_mut()).await?;
                    Ok(value)
                }
            }

            #[inline]
            async fn write_to<W: ::tokio::io::AsyncWrite + ::std::marker::Unpin>(&self, writer: &mut W) -> ::std::io::Result<()> {
                ::tokio::io::AsyncWriteExt::write_all(writer, self.bytes()).await?;
                Ok(())
            }

            #[inline(always)]
            fn bytes(&self) -> &[u8; ::std::mem::size_of::<Self>()] {
                unsafe { &*(self as *const Self as *const _) }
            }

            #[inline(always)]
            fn bytes_mut(&mut self) -> &mut [u8; ::std::mem::size_of::<Self>()] {
                unsafe { &mut *(self as *mut Self as *mut _) }
            }
        }
    }
}

pub fn print_bytes(bytes: &[u8]) {
    for (index, &byte) in bytes.iter().enumerate() {
        println!("0x{:04X}: 0x{:02X} == {:3} == {}", index, byte, byte, if byte.is_ascii_graphic() { byte as char } else { ' ' });
    }
}
