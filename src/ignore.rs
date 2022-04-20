use std::convert::Infallible;

use crate::de::{Map, Seq, Visitor, VisitorError};
use crate::error::Result;
use alloc::boxed::Box;

impl dyn Visitor<Error = Infallible> {
    pub fn ignore() -> &'static mut dyn Visitor<Error = Infallible> {
        static mut IGNORE: Ignore = Ignore;
        unsafe { &mut IGNORE }
        //
        // The following may be needed if stacked borrows gets more selective
        // about the above in the future:
        //
        //     unsafe { &mut *ptr::addr_of_mut!(IGNORE) }
        //
        // Conceptually we have an array of type [Ignore; ∞] in a static, which
        // is zero sized, and each caller of `fn ignore` gets a unique one of
        // them, as if by `&mut *ptr::addr_of_mut!(IGNORE[i++])` for some
        // appropriately synchronized i.
    }
}

pub(crate) struct Ignore;

impl VisitorError for Infallible {
    fn unexpected() -> Self {
        unreachable!()
    }
}

impl Visitor for Ignore {
    type Error = Infallible;
    fn raise(&mut self, _err: Self::Error) {}

    fn null(&mut self) {}

    fn boolean(&mut self, _b: bool) {}

    fn string(&mut self, _s: &str) {}

    fn negative(&mut self, _n: i64) {}

    fn nonnegative(&mut self, _n: u64) {}

    fn float(&mut self, _n: f64) {}

    fn seq(&mut self) -> Option<Box<dyn Seq<Self::Error> + '_>> {
        Some(Box::new(Ignore))
    }

    fn map(&mut self) -> Option<Box<dyn Map<Self::Error> + '_>> {
        Some(Box::new(Ignore))
    }
}

impl Seq<Infallible> for Ignore {
    fn element(&mut self) -> Result<&mut dyn Visitor<Error = Infallible>> {
        Ok(<dyn Visitor<Error = Infallible>>::ignore())
    }

    fn finish(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Map<Infallible> for Ignore {
    fn key(&mut self, _k: &str) -> Result<&mut dyn Visitor<Error = Infallible>> {
        Ok(<dyn Visitor<Error = Infallible>>::ignore())
    }

    fn finish(&mut self) -> Result<()> {
        Ok(())
    }
}
