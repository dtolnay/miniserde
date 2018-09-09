pub use std::borrow::Cow;
pub use std::boxed::Box;
pub use std::mem;
pub use std::option::Option::{self, None, Some};
pub use std::result::Result::{Err, Ok};
pub use std::string::String;

pub use self::help::Str as str;

mod help {
    pub type Str = str;
}
