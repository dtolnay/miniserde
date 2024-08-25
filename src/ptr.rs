use alloc::boxed::Box;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

// Like Box<T>, but holds NonNull<T> instead of Unique<T> to defer asserting
// uniqueness throughout the lifetime of the object until Drop is called. This
// makes it possible to take a &mut reference to the heap allocation, then move
// the NonuniqueBox, then write to the heap allocation through that old
// reference, then drop the NonuniqueBox.
pub struct NonuniqueBox<T>
where
    T: ?Sized,
{
    ptr: NonNull<T>,
}

impl<T> NonuniqueBox<T> {
    pub fn new(value: T) -> Self {
        NonuniqueBox::from(Box::new(value))
    }
}

impl<T> From<Box<T>> for NonuniqueBox<T>
where
    T: ?Sized,
{
    fn from(boxed: Box<T>) -> Self {
        let ptr = Box::into_raw(boxed);
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        NonuniqueBox { ptr }
    }
}

impl<T> Deref for NonuniqueBox<T>
where
    T: ?Sized,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for NonuniqueBox<T>
where
    T: ?Sized,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for NonuniqueBox<T>
where
    T: ?Sized,
{
    fn drop(&mut self) {
        let ptr = self.ptr.as_ptr();
        let _ = unsafe { Box::from_raw(ptr) };
    }
}
