#[macro_export]
macro_rules! write_raw {
    ($writer:expr, $raw:expr) => {
        $writer.write_all($raw.bytes()).await?;
    }
}


#[macro_export]
macro_rules! read_raw {
    ($reader:expr, $Raw:ty) => {
        {
            let mut raw = <$Raw as Default>::default();
            $reader.read_exact(raw.bytes_mut()).await?;
            raw
        }
    }
}

/// Promise that it is safe to transmute this type to and from its byte representation.
#[macro_export]
macro_rules! unsafe_impl_raw {
    ($T:ty) => {
        impl $T {
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
