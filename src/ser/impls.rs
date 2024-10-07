use crate::private;
use crate::ser::{Fragment, Map, Seq, Serialize};
use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::collections::{btree_map, BTreeMap};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::slice;
use core::str;
#[cfg(feature = "std")]
use std::collections::{hash_map, HashMap};
#[cfg(feature = "std")]
use std::hash::{BuildHasher, Hash};

impl Serialize for () {
    fn begin(&self) -> Fragment {
        Fragment::Null
    }
}

impl Serialize for bool {
    fn begin(&self) -> Fragment {
        Fragment::Bool(*self)
    }
}

impl Serialize for str {
    fn begin(&self) -> Fragment {
        Fragment::Str(Cow::Borrowed(self))
    }
}

impl Serialize for String {
    fn begin(&self) -> Fragment {
        Fragment::Str(Cow::Borrowed(self))
    }
}

macro_rules! unsigned {
    ($ty:ident) => {
        impl Serialize for $ty {
            fn begin(&self) -> Fragment {
                Fragment::U64(*self as u64)
            }
        }
    };
}
unsigned!(u8);
unsigned!(u16);
unsigned!(u32);
unsigned!(u64);
unsigned!(usize);

macro_rules! signed {
    ($ty:ident) => {
        impl Serialize for $ty {
            fn begin(&self) -> Fragment {
                Fragment::I64(*self as i64)
            }
        }
    };
}
signed!(i8);
signed!(i16);
signed!(i32);
signed!(i64);
signed!(isize);

macro_rules! float {
    ($ty:ident) => {
        impl Serialize for $ty {
            fn begin(&self) -> Fragment {
                Fragment::F64(*self as f64)
            }
        }
    };
}
float!(f32);
float!(f64);

impl<T> Serialize for &T
where
    T: ?Sized + Serialize,
{
    fn begin(&self) -> Fragment {
        (**self).begin()
    }
}

impl<T> Serialize for Box<T>
where
    T: ?Sized + Serialize,
{
    fn begin(&self) -> Fragment {
        (**self).begin()
    }
}

impl<T> Serialize for Option<T>
where
    T: Serialize,
{
    fn begin(&self) -> Fragment {
        match self {
            Some(some) => some.begin(),
            None => Fragment::Null,
        }
    }
}

impl<'a, T> Serialize for Cow<'a, T>
where
    T: ?Sized + ToOwned + Serialize,
{
    fn begin(&self) -> Fragment {
        (**self).begin()
    }
}

impl<A, B> Serialize for (A, B)
where
    A: Serialize,
    B: Serialize,
{
    fn begin(&self) -> Fragment {
        struct TupleStream<'a> {
            first: &'a dyn Serialize,
            second: &'a dyn Serialize,
            state: usize,
        }

        impl<'a> Seq for TupleStream<'a> {
            fn next(&mut self) -> Option<&dyn Serialize> {
                let state = self.state;
                self.state += 1;
                match state {
                    0 => Some(self.first),
                    1 => Some(self.second),
                    _ => None,
                }
            }
        }

        Fragment::Seq(Box::new(TupleStream {
            first: &self.0,
            second: &self.1,
            state: 0,
        }))
    }
}

impl<T> Serialize for [T]
where
    T: Serialize,
{
    fn begin(&self) -> Fragment {
        private::stream_slice(self)
    }
}

impl<T, const N: usize> Serialize for [T; N]
where
    T: Serialize,
{
    fn begin(&self) -> Fragment {
        private::stream_slice(self)
    }
}

impl<T> Serialize for Vec<T>
where
    T: Serialize,
{
    fn begin(&self) -> Fragment {
        private::stream_slice(self)
    }
}

#[cfg(feature = "std")]
impl<K, V, H> Serialize for HashMap<K, V, H>
where
    K: Hash + Eq + ToString,
    V: Serialize,
    H: BuildHasher,
{
    fn begin(&self) -> Fragment {
        struct HashMapStream<'a, K: 'a, V: 'a>(hash_map::Iter<'a, K, V>);

        impl<'a, K, V> Map for HashMapStream<'a, K, V>
        where
            K: ToString,
            V: Serialize,
        {
            fn next(&mut self) -> Option<(Cow<str>, &dyn Serialize)> {
                let (k, v) = self.0.next()?;
                Some((Cow::Owned(k.to_string()), v as &dyn Serialize))
            }
        }

        Fragment::Map(Box::new(HashMapStream(self.iter())))
    }
}

impl<K, V> Serialize for BTreeMap<K, V>
where
    K: ToString,
    V: Serialize,
{
    fn begin(&self) -> Fragment {
        private::stream_btree_map(self)
    }
}

impl private {
    pub fn stream_slice<T>(slice: &[T]) -> Fragment
    where
        T: Serialize,
    {
        struct SliceStream<'a, T: 'a>(slice::Iter<'a, T>);

        impl<'a, T> Seq for SliceStream<'a, T>
        where
            T: Serialize,
        {
            fn next(&mut self) -> Option<&dyn Serialize> {
                let element = self.0.next()?;
                Some(element)
            }
        }

        Fragment::Seq(Box::new(SliceStream(slice.iter())))
    }

    pub fn stream_btree_map<K, V>(map: &BTreeMap<K, V>) -> Fragment
    where
        K: ToString,
        V: Serialize,
    {
        struct BTreeMapStream<'a, K, V>(btree_map::Iter<'a, K, V>)
        where
            K: 'a,
            V: 'a;

        impl<'a, K, V> Map for BTreeMapStream<'a, K, V>
        where
            K: ToString,
            V: Serialize,
        {
            fn next(&mut self) -> Option<(Cow<str>, &dyn Serialize)> {
                let (k, v) = self.0.next()?;
                Some((Cow::Owned(k.to_string()), v as &dyn Serialize))
            }
        }

        Fragment::Map(Box::new(BTreeMapStream(map.iter())))
    }
}
