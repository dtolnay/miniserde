use crate::de::{Map, Seq, Visitor};
use crate::error::Result;
use alloc::boxed::Box;
use core::ptr;

impl dyn Visitor {
    pub fn ignore() -> &'static mut dyn Visitor {
        static mut IGNORE: Ignore = Ignore;

        // Conceptually we have an array of type [Ignore; ∞] in a static, which
        // is zero sized, and each caller of `fn ignore` gets a unique one of
        // them, as if by `&mut *ptr::addr_of_mut!(IGNORE[i++])` for some
        // appropriately synchronized i.
        unsafe { &mut *ptr::addr_of_mut!(IGNORE) }
    }
}

pub(crate) struct Ignore;

impl Visitor for Ignore {
    fn null(&mut self) -> Result<()> {
        Ok(())
    }

    fn boolean(&mut self, _b: bool) -> Result<()> {
        Ok(())
    }

    fn string(&mut self, _s: &str) -> Result<()> {
        Ok(())
    }

    fn negative(&mut self, _n: i64) -> Result<()> {
        Ok(())
    }

    fn nonnegative(&mut self, _n: u64) -> Result<()> {
        Ok(())
    }

    fn float(&mut self, _n: f64) -> Result<()> {
        Ok(())
    }

    fn seq(&mut self) -> Result<Box<dyn Seq + '_>> {
        Ok(Box::new(Ignore))
    }

    fn map(&mut self) -> Result<Box<dyn Map + '_>> {
        Ok(Box::new(Ignore))
    }
}

impl Seq for Ignore {
    fn element(&mut self) -> Result<&mut dyn Visitor> {
        Ok(<dyn Visitor>::ignore())
    }

    fn finish(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Map for Ignore {
    fn key(&mut self, _k: &str) -> Result<&mut dyn Visitor> {
        Ok(<dyn Visitor>::ignore())
    }

    fn finish(&mut self) -> Result<()> {
        Ok(())
    }
}
