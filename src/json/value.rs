use crate::de::{Deserialize, Map, Seq, Visitor, VisitorError};
use crate::error::Result;
use crate::json::{Array, Number, Object};
use crate::place::Cell;
use crate::private;
use crate::ser::{Fragment, Serialize};
use crate::Place;
use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::string::String;
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
#[derive(Clone, Debug)]
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

impl Serialize for Value {
    fn begin(&self) -> Fragment {
        match self {
            Value::Null => Fragment::Null,
            Value::Bool(b) => Fragment::Bool(*b),
            Value::Number(Number::U64(n)) => Fragment::U64(*n),
            Value::Number(Number::I64(n)) => Fragment::I64(*n),
            Value::Number(Number::F64(n)) => Fragment::F64(*n),
            Value::String(s) => Fragment::Str(Cow::Borrowed(s)),
            Value::Array(array) => private::stream_slice(array),
            Value::Object(object) => private::stream_object(object),
        }
    }
}

impl<E> Deserialize<E> for Value
where
    E: VisitorError,
{
    fn begin(out: &mut Cell<Self, E>) -> &mut dyn Visitor<Error = E> {
        impl<E> Visitor for Place<Value, E>
        where
            E: VisitorError,
        {
            type Error = E;

            fn raise(&mut self, err: Self::Error) {
                self.out.err(err);
            }

            fn null(&mut self) {
                self.out.set(Value::Null);
            }

            fn boolean(&mut self, b: bool) {
                self.out.set(Value::Bool(b));
            }

            fn string(&mut self, s: &str) {
                self.out.set(Value::String(s.to_owned()));
            }

            fn negative(&mut self, n: i64) {
                self.out.set(Value::Number(Number::I64(n)));
            }

            fn nonnegative(&mut self, n: u64) {
                self.out.set(Value::Number(Number::U64(n)));
            }

            fn float(&mut self, n: f64) {
                self.out.set(Value::Number(Number::F64(n)));
            }

            fn seq(&mut self) -> Option<Box<dyn Seq<Self::Error> + '_>> {
                Some(Box::new(ArrayBuilder {
                    out: &mut self.out,
                    array: Array::new(),
                    element: Cell::Empty,
                }))
            }

            fn map(&mut self) -> Option<Box<dyn Map<Self::Error> + '_>> {
                Some(Box::new(ObjectBuilder {
                    out: &mut self.out,
                    object: Object::new(),
                    key: Cell::Empty,
                    value: Cell::Empty,
                }))
            }
        }

        struct ArrayBuilder<'a, E> {
            out: &'a mut Cell<Value, E>,
            array: Array,
            element: Cell<Value, E>,
        }

        impl<'a, E> ArrayBuilder<'a, E> {
            fn shift(&mut self) {
                if let Cell::Ok(e) = self.element.take() {
                    self.array.push(e);
                }
            }
        }

        impl<'a, E: VisitorError> Seq<E> for ArrayBuilder<'a, E> {
            fn element(&mut self) -> Result<&mut dyn Visitor<Error = E>> {
                self.shift();
                Ok(Deserialize::begin(&mut self.element))
            }

            fn finish(&mut self) -> Result<()> {
                self.shift();
                self.out
                    .set(Value::Array(mem::replace(&mut self.array, Array::new())));
                Ok(())
            }
        }

        struct ObjectBuilder<'a, E> {
            out: &'a mut Cell<Value, E>,
            object: Object,
            key: Cell<String, E>,
            value: Cell<Value, E>,
        }

        impl<'a, E> ObjectBuilder<'a, E> {
            fn shift(&mut self) {
                if let (Cell::Ok(k), Cell::Ok(v)) = (self.key.take(), self.value.take()) {
                    self.object.insert(k, v);
                }
            }
        }

        impl<'a, E: VisitorError> Map<E> for ObjectBuilder<'a, E> {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<Error = E>> {
                self.shift();
                self.key = Cell::Ok(k.to_owned());
                Ok(Deserialize::begin(&mut self.value))
            }

            fn finish(&mut self) -> Result<()> {
                self.shift();
                *self.out = Cell::Ok(Value::Object(mem::replace(&mut self.object, Object::new())));
                Ok(())
            }
        }

        Place::new(out)
    }
}
