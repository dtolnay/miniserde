use crate::de::{Deserialize, Map, Seq, Visitor, VisitorError};
use crate::error::{Error, Result};
use crate::ignore::Ignore;
use crate::place::Cell;
use crate::ptr::NonuniqueBox;
use crate::Place;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::{self, ManuallyDrop};
use core::str::{self, FromStr};
#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::hash::{BuildHasher, Hash};

impl<E> Deserialize<E> for ()
where
    E: VisitorError,
{
    fn begin(out: &mut Cell<Self, E>) -> &mut dyn Visitor<Error = E> {
        impl<E> Visitor for Place<(), E>
        where
            E: VisitorError,
        {
            type Error = E;
            fn raise(&mut self, err: Self::Error) {
                self.out = Cell::Err(err);
            }

            fn null(&mut self) {
                self.out.set(());
            }
        }
        Place::new(out)
    }
}

impl<E: VisitorError> Deserialize<E> for bool {
    fn begin(out: &mut Cell<Self, E>) -> &mut dyn Visitor<Error = E> {
        impl<E: VisitorError> Visitor for Place<bool, E> {
            type Error = E;

            fn raise(&mut self, err: E) {
                self.out.err(err);
            }
            fn boolean(&mut self, b: bool) {
                self.out.set(b);
            }
        }
        Place::new(out)
    }
}

impl<E: VisitorError> Deserialize<E> for String {
    fn begin(out: &mut Cell<Self, E>) -> &mut dyn Visitor<Error = E> {
        impl<E: VisitorError> Visitor for Place<String, E> {
            type Error = E;

            fn raise(&mut self, err: E) {
                self.out.err(err);
            }

            fn string(&mut self, s: &str) {
                self.out.set(s.to_owned());
            }
        }
        Place::new(out)
    }
}

macro_rules! signed {
    ($ty:ident) => {
        impl<E: VisitorError> Deserialize<E> for $ty {
            fn begin(out: &mut Cell<Self, E>) -> &mut dyn Visitor<Error = E> {
                impl<E: VisitorError> Visitor for Place<$ty, E> {
                    type Error = E;

                    fn raise(&mut self, err: E) {
                        self.out.err(err);
                    }

                    fn negative(&mut self, n: i64) {
                        if n >= $ty::min_value() as i64 {
                            self.out.set(n as $ty);
                        } else {
                            self.out.err(E::unexpected());
                        }
                    }

                    fn nonnegative(&mut self, n: u64) {
                        if n <= $ty::max_value() as u64 {
                            self.out.set(n as $ty);
                        } else {
                            self.out.err(E::unexpected());
                        }
                    }
                }
                Place::new(out)
            }
        }
    };
}
signed!(i8);
signed!(i16);
signed!(i32);
signed!(i64);
signed!(isize);

macro_rules! unsigned {
    ($ty:ident) => {
        impl<E: VisitorError> Deserialize<E> for $ty {
            fn begin(out: &mut Cell<Self, E>) -> &mut dyn Visitor<Error = E> {
                impl<E: VisitorError> Visitor for Place<$ty, E> {
                    type Error = E;

                    fn raise(&mut self, err: E) {
                        self.out.err(err);
                    }

                    fn nonnegative(&mut self, n: u64) {
                        if n <= $ty::max_value() as u64 {
                            self.out.set(n as $ty);
                        } else {
                            self.out.err(E::unexpected());
                        }
                    }
                }
                Place::new(out)
            }
        }
    };
}
unsigned!(u8);
unsigned!(u16);
unsigned!(u32);
unsigned!(u64);
unsigned!(usize);

macro_rules! float {
    ($ty:ident) => {
        impl<E: VisitorError> Deserialize<E> for $ty {
            fn begin(out: &mut Cell<Self, E>) -> &mut dyn Visitor<Error = E> {
                impl<E: VisitorError> Visitor for Place<$ty, E> {
                    type Error = E;

                    fn raise(&mut self, err: E) {
                        self.out.err(err);
                    }

                    fn negative(&mut self, n: i64) {
                        self.out.set(n as $ty);
                    }

                    fn nonnegative(&mut self, n: u64) {
                        self.out.set(n as $ty);
                    }

                    fn float(&mut self, n: f64) {
                        self.out.set(n as $ty);
                    }
                }
                Place::new(out)
            }
        }
    };
}
float!(f32);
float!(f64);

impl<E: VisitorError, T: Deserialize<E>> Deserialize<E> for Box<T> {
    fn begin(out: &mut Cell<Self, E>) -> &mut dyn Visitor<Error = E> {
        impl<E: VisitorError, T: Deserialize<E>> Visitor for Place<Box<T>, E> {
            type Error = E;

            fn raise(&mut self, err: E) {
                self.out.err(err);
            }

            fn null(&mut self) {
                let mut out = Cell::Empty;
                Deserialize::begin(&mut out).null();
                self.out = out.map(Box::new);
            }

            fn boolean(&mut self, b: bool) {
                let mut out = Cell::Empty;
                Deserialize::begin(&mut out).boolean(b);
                self.out = out.map(Box::new);
            }

            fn string(&mut self, s: &str) {
                let mut out = Cell::Empty;
                Deserialize::begin(&mut out).string(s);
                self.out = out.map(Box::new);
            }

            fn negative(&mut self, n: i64) {
                let mut out = Cell::Empty;
                Deserialize::begin(&mut out).negative(n);
                self.out = out.map(Box::new);
            }

            fn nonnegative(&mut self, n: u64) {
                let mut out = Cell::Empty;
                Deserialize::begin(&mut out).nonnegative(n);
                self.out = out.map(Box::new);
            }

            fn float(&mut self, n: f64) {
                let mut out = Cell::Empty;
                Deserialize::begin(&mut out).float(n);
                self.out = out.map(Box::new);
            }

            fn seq(&mut self) -> Option<Box<dyn Seq<E> + '_>> {
                let mut value = NonuniqueBox::new(Cell::Empty);
                let ptr = unsafe { extend_lifetime!(&mut *value as &mut Option<T>) };
                if !self.out.is_err() {
                    Some(Box::new(BoxSeq {
                        out: &mut self.out,
                        value,
                        seq: ManuallyDrop::new(Deserialize::begin(ptr).seq()?),
                    }))
                } else {
                    None
                }
            }

            fn map(&mut self) -> Option<Box<dyn Map<E> + '_>> {
                let mut value = NonuniqueBox::new(Cell::Empty);
                let ptr = unsafe { extend_lifetime!(&mut *value as &mut Option<T>) };
                if !self.out.is_err() {
                    Some(Box::new(BoxMap {
                        out: &mut self.out,
                        value,
                        map: ManuallyDrop::new(Deserialize::begin(ptr).map()?),
                    }))
                } else {
                    None
                }
            }
        }

        struct BoxSeq<'a, T: 'a, E> {
            out: &'a mut Cell<Box<T>, E>,
            value: NonuniqueBox<Cell<T, E>>,
            // May borrow from self.value, so must drop first.
            seq: ManuallyDrop<Box<dyn Seq<E> + 'a>>,
        }

        impl<'a, T: 'a, E> Drop for BoxSeq<'a, T, E> {
            fn drop(&mut self) {
                unsafe { ManuallyDrop::drop(&mut self.seq) }
            }
        }

        impl<'a, E: VisitorError, T: Deserialize<E>> Seq<E> for BoxSeq<'a, T, E> {
            fn element(&mut self) -> Result<&mut dyn Visitor<Error = E>> {
                self.seq.element()
            }

            fn finish(&mut self) -> Result<()> {
                self.seq.finish()?;
                // ? why is this done?
                todo!("*self.seq = Box::new(Ignore)");
                *self.out = self.value.take().map(Box::new);
                Ok(())
            }
        }

        struct BoxMap<'a, T: 'a, E> {
            out: &'a mut Cell<Box<T>, E>,
            value: NonuniqueBox<Cell<T, E>>,
            // May borrow from self.value, so must drop first.
            map: ManuallyDrop<Box<dyn Map<E> + 'a>>,
        }

        impl<'a, T: 'a, E> Drop for BoxMap<'a, T, E> {
            fn drop(&mut self) {
                unsafe { ManuallyDrop::drop(&mut self.map) }
            }
        }

        impl<'a, E: VisitorError, T: Deserialize<E>> Map<E> for BoxMap<'a, T, E> {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<Error = E>> {
                self.map.key(k)
            }

            fn finish(&mut self) -> Result<()> {
                self.map.finish()?;
                *self.map = Box::new(Ignore);
                *self.out = Some(Box::new(self.value.take().unwrap()));
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl<E: VisitorError, T: Deserialize<E>> Deserialize<E> for Option<T> {
    #[inline]
    fn default() -> Option<Self> {
        Some(None)
    }
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<Error = E> {
        impl<E: VisitorError, T: Deserialize<E>> Visitor for Place<Option<T>, E> {
            fn null(&mut self) -> Result<()> {
                self.out = Some(None);
                Ok(())
            }

            fn boolean(&mut self, b: bool) {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).boolean(b)
            }

            fn string(&mut self, s: &str) {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).string(s)
            }

            fn negative(&mut self, n: i64) {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).negative(n)
            }

            fn nonnegative(&mut self, n: u64) {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).nonnegative(n)
            }

            fn float(&mut self, n: f64) {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).float(n)
            }

            fn seq(&mut self) -> Option<Box<dyn Seq<E> + '_>> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).seq()
            }

            fn map(&mut self) -> Option<Box<dyn Map<E> + '_>> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).map()
            }
        }

        Place::new(out)
    }
}

impl<E: VisitorError, A: Deserialize<E>, B: Deserialize<E>> Deserialize<E> for (A, B) {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<Error = E> {
        impl<E: VisitorError, A: Deserialize<E>, B: Deserialize<E>> Visitor for Place<(A, B), E> {
            fn seq(&mut self) -> Result<Box<dyn Seq<E> + '_>> {
                Ok(Box::new(TupleBuilder {
                    out: &mut self.out,
                    tuple: (None, None),
                }))
            }
        }

        struct TupleBuilder<'a, A: 'a, B: 'a> {
            out: &'a mut Option<(A, B)>,
            tuple: (Option<A>, Option<B>),
        }

        impl<'a, E: VisitorError, A: Deserialize<E>, B: Deserialize<E>> Seq<E> for TupleBuilder<'a, A, B> {
            fn element(&mut self) -> Result<&mut dyn Visitor<Error = E>> {
                if self.tuple.0.is_none() {
                    Ok(Deserialize::begin(&mut self.tuple.0))
                } else if self.tuple.1.is_none() {
                    Ok(Deserialize::begin(&mut self.tuple.1))
                } else {
                    Err(Error)
                }
            }

            fn finish(&mut self) -> Result<()> {
                if let (Some(a), Some(b)) = (self.tuple.0.take(), self.tuple.1.take()) {
                    *self.out = Some((a, b));
                    Ok(())
                } else {
                    Err(Error)
                }
            }
        }

        Place::new(out)
    }
}

impl<E: VisitorError, T: Deserialize<E>> Deserialize<E> for Vec<T> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<Error = E> {
        impl<E: VisitorError, T: Deserialize<E>> Visitor for Place<Vec<T>, E> {
            fn seq(&mut self) -> Result<Box<dyn Seq<E> + '_>> {
                Ok(Box::new(VecBuilder {
                    out: &mut self.out,
                    vec: Vec::new(),
                    element: None,
                }))
            }
        }

        struct VecBuilder<'a, T: 'a, E> {
            out: &'a mut Cell<Vec<T>, E>,
            vec: Vec<T>,
            element: Cell<T, E>,
        }

        impl<'a, T, E> VecBuilder<'a, T, E> {
            fn shift(&mut self) {
                match self.element.take() {
                    Cell::Ok(e) => {
                        self.vec.push(e);
                    }
                    Cell::Err(e) => {
                        *self.out = Cell::Err(e);
                    }
                    _ => (),
                }
            }
        }

        impl<'a, E: VisitorError, T: Deserialize<E>> Seq<E> for VecBuilder<'a, T, E> {
            fn element(&mut self) -> Result<&mut dyn Visitor<Error = E>> {
                self.shift();
                Ok(Deserialize::begin(&mut self.element))
            }

            fn finish(&mut self) -> Result<()> {
                self.shift();
                *self.out = Some(mem::replace(&mut self.vec, Vec::new()));
                Ok(())
            }
        }

        Place::new(out)
    }
}

#[cfg(feature = "std")]
impl<E, K, V, H> Deserialize<E> for HashMap<K, V, H>
where
    K: FromStr + Hash + Eq,
    V: Deserialize<E>,
    H: BuildHasher + Default,
    E: VisitorError,
{
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<Error = E> {
        impl<E, K, V, H> Visitor for Place<HashMap<K, V, H>, E>
        where
            K: FromStr + Hash + Eq,
            V: Deserialize<E>,
            H: BuildHasher + Default,
            E: VisitorError,
        {
            fn map(&mut self) -> Option<Box<dyn Map<E> + '_>> {
                Some(Box::new(MapBuilder {
                    out: &mut self.out,
                    map: HashMap::with_hasher(H::default()),
                    key: Cell::Empty,
                    value: Cell::Empty,
                }))
            }
        }

        struct MapBuilder<'a, K: 'a, V: 'a, H: 'a, E> {
            out: &'a mut Cell<HashMap<K, V, H>, E>,
            map: HashMap<K, V, H>,
            key: Cell<K, E>,
            value: Cell<V, E>,
        }

        impl<'a, K: Hash + Eq, V, H: BuildHasher, E> MapBuilder<'a, K, V, H, E> {
            fn shift(&mut self) {
                if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                    self.map.insert(k, v);
                }
            }
        }

        impl<'a, E, K, V, H> Map<E> for MapBuilder<'a, K, V, H, E>
        where
            K: FromStr + Hash + Eq,
            V: Deserialize<E>,
            H: BuildHasher + Default,
            E: VisitorError,
        {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<Error = E>> {
                self.shift();
                self.key = Some(match K::from_str(k) {
                    Ok(key) => key,
                    Err(_) => return Err(Error),
                });
                Ok(Deserialize::begin(&mut self.value))
            }

            fn finish(&mut self) -> Result<()> {
                self.shift();
                let substitute = HashMap::with_hasher(H::default());
                *self.out = Some(mem::replace(&mut self.map, substitute));
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl<E: VisitorError, K: FromStr + Ord, V: Deserialize<E>> Deserialize<E> for BTreeMap<K, V> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<Error = E> {
        impl<E: VisitorError, K: FromStr + Ord, V: Deserialize<E>> Visitor for Place<BTreeMap<K, V>, E> {
            fn map(&mut self) -> Result<Box<dyn Map<E> + '_>> {
                Ok(Box::new(MapBuilder {
                    out: &mut self.out,
                    map: BTreeMap::new(),
                    key: Cell::Empty,
                    value: Cell::Empty,
                }))
            }
        }

        struct MapBuilder<'a, K: 'a, V: 'a, E> {
            out: &'a mut Option<BTreeMap<K, V>>,
            map: BTreeMap<K, V>,
            key: Cell<K, E>,
            value: Cell<V, E>,
        }

        impl<'a, K: Ord, V, E> MapBuilder<'a, K, V, E> {
            fn shift(&mut self) {
                if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                    self.map.insert(k, v);
                }
            }
        }

        impl<'a, E: VisitorError, K: FromStr + Ord, V: Deserialize<E>> Map<E> for MapBuilder<'a, K, V, E> {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<Error = E>> {
                self.shift();
                self.key = Some(match K::from_str(k) {
                    Ok(key) => key,
                    Err(_) => return Err(Error),
                });
                Ok(Deserialize::begin(&mut self.value))
            }

            fn finish(&mut self) -> Result<()> {
                self.shift();
                *self.out = Some(mem::replace(&mut self.map, BTreeMap::new()));
                Ok(())
            }
        }

        Place::new(out)
    }
}
