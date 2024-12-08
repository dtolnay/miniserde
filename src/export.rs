#![allow(non_camel_case_types)]

#[doc(hidden)]
pub use core::option::Option::{None, Some};
#[doc(hidden)]
pub use core::result::Result::{Err, Ok};
#[doc(hidden)]
pub use core::unreachable;

#[doc(hidden)]
pub type Box<T> = alloc::boxed::Box<T>;
#[doc(hidden)]
pub type Cow<'a, T> = alloc::borrow::Cow<'a, T>;
#[doc(hidden)]
pub type Option<T> = core::option::Option<T>;
#[doc(hidden)]
pub type String = alloc::string::String;
#[doc(hidden)]
pub type str = core::primitive::str;
#[doc(hidden)]
pub type usize = core::primitive::usize;
