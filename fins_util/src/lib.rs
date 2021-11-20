mod num;
mod raw;

pub use num::*;
pub use raw::*;

/// For when you need to use `?` without the `Into` conversion in a const fn.
#[macro_export]
macro_rules! try_const {
    ($e:expr) => {
        match $e {
            Ok(value) => value,
            Err(error) => return Err(error),
        }
    };
}
