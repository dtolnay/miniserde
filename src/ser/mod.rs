//! Serialization traits.
//!
//! Serialization in miniserde works by traversing an input object and
//! decomposing it iteratively into a stream of fragments.
//!
//! ## Serializing a primitive
//!
//! ```rust
//! use miniserde::ser::{Fragment, Serialize};
//!
//! // The data structure that we want to serialize as a primitive.
//! struct MyBoolean(bool);
//!
//! impl Serialize for MyBoolean {
//!     fn begin(&self) -> Fragment {
//!         Fragment::Bool(self.0)
//!     }
//! }
//! ```
//!
//! ## Serializing a sequence
//!
//! ```rust
//! use miniserde::ser::{Fragment, Seq, Serialize};
//!
//! // Some custom sequence type that we want to serialize.
//! struct MyVec<T>(Vec<T>);
//!
//! impl<T> Serialize for MyVec<T>
//! where
//!     T: Serialize,
//! {
//!     fn begin(&self) -> Fragment {
//!         Fragment::Seq(Box::new(SliceStream { iter: self.0.iter() }))
//!     }
//! }
//!
//! struct SliceStream<'a, T: 'a> {
//!     iter: std::slice::Iter<'a, T>,
//! }
//!
//! impl<'a, T> Seq for SliceStream<'a, T>
//! where
//!     T: Serialize,
//! {
//!     fn next(&mut self) -> Option<&dyn Serialize> {
//!         let element = self.iter.next()?;
//!         Some(element)
//!     }
//! }
//! ```
//!
//! ## Serializing a map or struct
//!
//! This code demonstrates what is generated for structs by
//! `#[derive(Serialize)]`.
//!
//! ```rust
//! use miniserde::ser::{Fragment, Map, Serialize};
//! use std::borrow::Cow;
//!
//! // The struct that we would like to serialize.
//! struct Demo {
//!     code: u32,
//!     message: String,
//! }
//!
//! impl Serialize for Demo {
//!     fn begin(&self) -> Fragment {
//!         Fragment::Map(Box::new(DemoStream {
//!             data: self,
//!             state: 0,
//!         }))
//!     }
//! }
//!
//! struct DemoStream<'a> {
//!     data: &'a Demo,
//!     state: usize,
//! }
//!
//! impl<'a> Map for DemoStream<'a> {
//!     fn next(&mut self) -> Option<(Cow<str>, &dyn Serialize)> {
//!         let state = self.state;
//!         self.state += 1;
//!         match state {
//!             0 => Some((Cow::Borrowed("code"), &self.data.code)),
//!             1 => Some((Cow::Borrowed("message"), &self.data.message)),
//!             _ => None,
//!         }
//!     }
//! }
//! ```

mod impls;

use alloc::borrow::Cow;
use alloc::boxed::Box;

/// One unit of output produced during serialization.
///
/// [Refer to the module documentation for examples.][crate::ser]
pub enum Fragment<'a> {
    Null,
    Bool(bool),
    Str(Cow<'a, str>),
    U64(u64),
    I64(i64),
    F64(f64),
    Seq(Box<dyn Seq + 'a>),
    Map(Box<dyn Map + 'a>),
}

/// Trait for data structures that can be serialized to a JSON string.
///
/// [Refer to the module documentation for examples.][crate::ser]
pub trait Serialize {
    fn begin(&self) -> Fragment;
}

/// Trait that can iterate elements of a sequence.
///
/// [Refer to the module documentation for examples.][crate::ser]
pub trait Seq {
    fn next(&mut self) -> Option<&dyn Serialize>;
}

/// Trait that can iterate key-value entries of a map or struct.
///
/// [Refer to the module documentation for examples.][crate::ser]
pub trait Map {
    fn next(&mut self) -> Option<(Cow<str>, &dyn Serialize)>;
}
