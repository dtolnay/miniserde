use crate::de::{Deserialize, Map, Visitor};
use crate::error::Result;
use crate::json::{drop, Value};
use crate::ser::{self, Fragment, Serialize};
use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::collections::{btree_map, BTreeMap};
use alloc::string::String;
use core::fmt::{self, Debug};
use core::mem::{self, ManuallyDrop};
use core::ops::{Deref, DerefMut};
use core::ptr;
use core::str;

/// A `BTreeMap<String, Value>` with a non-recursive drop impl.
#[derive(Clone, Default)]
pub struct Object {
    inner: BTreeMap<String, Value>,
}

impl Drop for Object {
    fn drop(&mut self) {
        for (_, child) in mem::take(&mut self.inner) {
            drop::safely(child);
        }
    }
}

fn take(object: Object) -> BTreeMap<String, Value> {
    let object = ManuallyDrop::new(object);
    unsafe { ptr::read(&object.inner) }
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

impl Debug for Object {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Object ")?;
        formatter.debug_map().entries(self).finish()
    }
}

impl Serialize for Object {
    fn begin(&self) -> Fragment {
        struct ObjectIter<'a>(btree_map::Iter<'a, String, Value>);

        impl<'a> ser::Map for ObjectIter<'a> {
            fn next(&mut self) -> Option<(Cow<str>, &dyn Serialize)> {
                let (k, v) = self.0.next()?;
                Some((Cow::Borrowed(k), v as &dyn Serialize))
            }
        }

        Fragment::Map(Box::new(ObjectIter(self.iter())))
    }
}

impl Deserialize for Object {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        make_place!(Place);

        impl Visitor for Place<Object> {
            fn map(&mut self) -> Result<Box<dyn Map + '_>> {
                Ok(Box::new(ObjectBuilder {
                    out: &mut self.out,
                    object: Object::new(),
                    key: None,
                    value: None,
                }))
            }
        }

        struct ObjectBuilder<'a> {
            out: &'a mut Option<Object>,
            object: Object,
            key: Option<String>,
            value: Option<Value>,
        }

        impl<'a> ObjectBuilder<'a> {
            fn shift(&mut self) {
                if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                    self.object.insert(k, v);
                }
            }
        }

        impl<'a> Map for ObjectBuilder<'a> {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor> {
                self.shift();
                self.key = Some(k.to_owned());
                Ok(Deserialize::begin(&mut self.value))
            }

            fn finish(&mut self) -> Result<()> {
                self.shift();
                *self.out = Some(mem::replace(&mut self.object, Object::new()));
                Ok(())
            }
        }

        Place::new(out)
    }
}
