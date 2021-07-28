pub use crate::lib::Box;
pub use crate::lib::Cow;

#[cfg(not(feature = "std"))]
pub use core::option::Option::{self, None, Some};
#[cfg(feature = "std")]
pub use std::option::Option::{self, None, Some};

#[cfg(not(feature = "std"))]
pub use core::result::Result::{Err, Ok};
#[cfg(feature = "std")]
pub use std::result::Result::{Err, Ok};

pub use crate::lib::String;

pub use self::help::Str as str;
pub use self::help::Usize as usize;

mod help {
    pub type Str = str;
    pub type Usize = usize;
}
