#[cfg(not(feature = "std"))]
pub use alloc::borrow::Cow;
#[cfg(feature = "std")]
pub use std::borrow::Cow;

#[cfg(not(feature = "std"))]
pub use alloc::boxed::Box;
#[cfg(feature = "std")]
pub use std::boxed::Box;

#[cfg(not(feature = "std"))]
pub use core::option::Option::{self, None, Some};
#[cfg(feature = "std")]
pub use std::option::Option::{self, None, Some};

#[cfg(not(feature = "std"))]
pub use core::result::Result::{Err, Ok};
#[cfg(feature = "std")]
pub use std::result::Result::{Err, Ok};

#[cfg(not(feature = "std"))]
pub use alloc::string::String;
#[cfg(feature = "std")]
pub use std::string::String;

pub use self::help::Str as str;
pub use self::help::Usize as usize;

mod help {
    pub type Str = str;
    pub type Usize = usize;
}
