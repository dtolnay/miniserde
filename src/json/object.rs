use std::borrow::Cow;
use std::collections::{btree_map, BTreeMap};
use std::iter::FromIterator;
use std::mem;
use std::ops::{Deref, DerefMut};

use crate::json::{drop, Value};
use crate::private;
use crate::ser::{self, Fragment, Serialize};

/// A `BTreeMap<String, Value>` with a non-recursive drop impl.
#[derive(Clone, Debug, Default)]
pub struct Object {
    inner: BTreeMap<String, Value>,
}

impl Drop for Object {
    fn drop(&mut self) {
        for (_, child) in mem::replace(&mut self.inner, BTreeMap::new()) {
            drop::safely(child);
        }
    }
}

fn take(mut object: Object) -> BTreeMap<String, Value> {
    unsafe {
        let inner = mem::replace(&mut object.inner, mem::uninitialized());
        mem::forget(object);
        inner
    }
}

impl Object {
    pub fn new() -> Self {
        Object {
            inner: BTreeMap::new(),
        }
    }
}

impl Deref for Object {
    type Target = BTreeMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Object {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl IntoIterator for Object {
    type Item = (String, Value);
    type IntoIter = <BTreeMap<String, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        take(self).into_iter()
    }
}

impl<'a> IntoIterator for &'a Object {
    type Item = (&'a String, &'a Value);
    type IntoIter = <&'a BTreeMap<String, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Object {
    type Item = (&'a String, &'a mut Value);
    type IntoIter = <&'a mut BTreeMap<String, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl FromIterator<(String, Value)> for Object {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (String, Value)>,
    {
        Object {
            inner: BTreeMap::from_iter(iter),
        }
    }
}

impl private {
    pub fn stream_object(object: &Object) -> Fragment {
        struct ObjectIter<'a>(btree_map::Iter<'a, String, Value>);

        impl<'a> ser::Map for ObjectIter<'a> {
            fn next(&mut self) -> Option<(Cow<str>, &dyn Serialize)> {
                let (k, v) = self.0.next()?;
                Some((Cow::Borrowed(k), v as &dyn Serialize))
            }
        }

        Fragment::Map(Box::new(ObjectIter(object.iter())))
    }
}
