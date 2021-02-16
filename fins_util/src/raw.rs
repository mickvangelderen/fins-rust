#[macro_export]
macro_rules! unsafe_impl_raw {
    ($T:ty) => {
        impl $T {
            #[inline(always)]
            fn uninit() -> Self {
                unsafe { std::mem::MaybeUninit::uninit().assume_init() }
            }

            #[inline(always)]
            fn as_bytes(&self) -> &[u8; std::mem::size_of::<Self>()] {
                unsafe { &*(self as *const Self as *const _) }
            }
        
            #[inline(always)]
            fn bytes_mut(&mut self) -> &mut [u8; std::mem::size_of::<Self>()] {
                unsafe { &mut*(self as *mut Self as *mut _) }
            }
        }
    }
}

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
