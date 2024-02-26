use crate::de::{Deserialize, Map, Seq, Visitor};
use crate::error::Result;
use crate::json::{Array, Number, Object};
use crate::ser::{Fragment, Serialize};
use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::{self, Debug};
use core::mem;
use core::str;

/// Any valid JSON value.
///
/// This type has a non-recursive drop implementation so it is safe to build
/// arbitrarily deeply nested instances.
///
/// ```rust
/// use miniserde::json::{Array, Value};
///
/// let mut value = Value::Null;
#[cfg_attr(not(miri), doc = "for _ in 0..100000 {")]
#[cfg_attr(miri, doc = "for _ in 0..40 {")]
///     let mut array = Array::new();
///     array.push(value);
///     value = Value::Array(array);
/// }
/// // no stack overflow when `value` goes out of scope
/// ```
#[derive(Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Array),
    Object(Object),
}

impl Default for Value {
    /// The default value is null.
    fn default() -> Self {
        Value::Null
    }
}

impl Debug for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Null => formatter.write_str("Null"),
            Value::Bool(boolean) => write!(formatter, "Bool({})", boolean),
            Value::Number(number) => write!(formatter, "Number({})", number),
            Value::String(string) => write!(formatter, "String({:?})", string),
            Value::Array(array) => Debug::fmt(array, formatter),
            Value::Object(object) => Debug::fmt(object, formatter),
        }
    }
}

impl Serialize for Value {
    fn begin(&self) -> Fragment {
        match self {
            Value::Null => Fragment::Null,
            Value::Bool(b) => Fragment::Bool(*b),
            Value::Number(number) => Serialize::begin(number),
            Value::String(s) => Fragment::Str(Cow::Borrowed(s)),
            Value::Array(array) => Serialize::begin(array),
            Value::Object(object) => Serialize::begin(object),
        }
    }
}

impl Deserialize for Value {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        make_place!(Place);

        impl Visitor for Place<Value> {
            fn null(&mut self) -> Result<()> {
                self.out = Some(Value::Null);
                Ok(())
            }

            fn boolean(&mut self, b: bool) -> Result<()> {
                self.out = Some(Value::Bool(b));
                Ok(())
            }

            fn string(&mut self, s: &str) -> Result<()> {
                self.out = Some(Value::String(s.to_owned()));
                Ok(())
            }

            fn negative(&mut self, n: i64) -> Result<()> {
                self.out = Some(Value::Number(Number::I64(n)));
                Ok(())
            }

            fn nonnegative(&mut self, n: u64) -> Result<()> {
                self.out = Some(Value::Number(Number::U64(n)));
                Ok(())
            }

            fn float(&mut self, n: f64) -> Result<()> {
                self.out = Some(Value::Number(Number::F64(n)));
                Ok(())
            }

            fn seq(&mut self) -> Result<Box<dyn Seq + '_>> {
                Ok(Box::new(ArrayBuilder {
                    out: &mut self.out,
                    array: Array::new(),
                    element: None,
                }))
            }

            fn map(&mut self) -> Result<Box<dyn Map + '_>> {
                Ok(Box::new(ObjectBuilder {
                    out: &mut self.out,
                    object: Object::new(),
                    key: None,
                    value: None,
                }))
            }
        }

        struct ArrayBuilder<'a> {
            out: &'a mut Option<Value>,
            array: Array,
            element: Option<Value>,
        }

        impl<'a> ArrayBuilder<'a> {
            fn shift(&mut self) {
                if let Some(e) = self.element.take() {
                    self.array.push(e);
                }
            }
        }

        impl<'a> Seq for ArrayBuilder<'a> {
            fn element(&mut self) -> Result<&mut dyn Visitor> {
                self.shift();
                Ok(Deserialize::begin(&mut self.element))
            }

            fn finish(&mut self) -> Result<()> {
                self.shift();
                *self.out = Some(Value::Array(mem::replace(&mut self.array, Array::new())));
                Ok(())
            }
        }

        struct ObjectBuilder<'a> {
            out: &'a mut Option<Value>,
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
                *self.out = Some(Value::Object(mem::replace(&mut self.object, Object::new())));
                Ok(())
            }
        }

        Place::new(out)
    }
}
