use crate::de::{Deserialize, Map, Seq, Visitor};
use crate::error::{Error, Result};
use crate::ignore::Ignore;
use crate::ptr::NonuniqueBox;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::mem::{self, ManuallyDrop, MaybeUninit};
use core::ptr;
use core::str::{self, FromStr};
#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::hash::{BuildHasher, Hash};

impl Deserialize for () {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        make_place!(Place);

        impl Visitor for Place<()> {
            fn null(&mut self) -> Result<()> {
                self.out = Some(());
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl Deserialize for bool {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        make_place!(Place);

        impl Visitor for Place<bool> {
            fn boolean(&mut self, b: bool) -> Result<()> {
                self.out = Some(b);
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl Deserialize for String {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        make_place!(Place);

        impl Visitor for Place<String> {
            fn string(&mut self, s: &str) -> Result<()> {
                self.out = Some(s.to_owned());
                Ok(())
            }
        }

        Place::new(out)
    }
}

macro_rules! signed {
    ($ty:ident) => {
        impl Deserialize for $ty {
            fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
                make_place!(Place);

                impl Visitor for Place<$ty> {
                    fn negative(&mut self, n: i64) -> Result<()> {
                        if n >= $ty::MIN as i64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(Error)
                        }
                    }

                    fn nonnegative(&mut self, n: u64) -> Result<()> {
                        if n <= $ty::MAX as u64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(Error)
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
        impl Deserialize for $ty {
            fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
                make_place!(Place);

                impl Visitor for Place<$ty> {
                    fn nonnegative(&mut self, n: u64) -> Result<()> {
                        if n <= $ty::MAX as u64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(Error)
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
        impl Deserialize for $ty {
            fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
                make_place!(Place);

                impl Visitor for Place<$ty> {
                    fn negative(&mut self, n: i64) -> Result<()> {
                        self.out = Some(n as $ty);
                        Ok(())
                    }

                    fn nonnegative(&mut self, n: u64) -> Result<()> {
                        self.out = Some(n as $ty);
                        Ok(())
                    }

                    fn float(&mut self, n: f64) -> Result<()> {
                        self.out = Some(n as $ty);
                        Ok(())
                    }
                }

                Place::new(out)
            }
        }
    };
}
float!(f32);
float!(f64);

impl<T> Deserialize for Box<T>
where
    T: Deserialize,
{
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        make_place!(Place);

        impl<T> Visitor for Place<Box<T>>
        where
            T: Deserialize,
        {
            fn null(&mut self) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).null()?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn boolean(&mut self, b: bool) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).boolean(b)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn string(&mut self, s: &str) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).string(s)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn negative(&mut self, n: i64) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).negative(n)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn nonnegative(&mut self, n: u64) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).nonnegative(n)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn float(&mut self, n: f64) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).float(n)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn seq(&mut self) -> Result<Box<dyn Seq + '_>> {
                let mut value = NonuniqueBox::new(None);
                let ptr = unsafe { extend_lifetime!(&mut *value as &mut Option<T>) };
                Ok(Box::new(BoxSeq {
                    out: &mut self.out,
                    value,
                    seq: ManuallyDrop::new(Deserialize::begin(ptr).seq()?),
                }))
            }

            fn map(&mut self) -> Result<Box<dyn Map + '_>> {
                let mut value = NonuniqueBox::new(None);
                let ptr = unsafe { extend_lifetime!(&mut *value as &mut Option<T>) };
                Ok(Box::new(BoxMap {
                    out: &mut self.out,
                    value,
                    map: ManuallyDrop::new(Deserialize::begin(ptr).map()?),
                }))
            }
        }

        struct BoxSeq<'a, T: 'a> {
            out: &'a mut Option<Box<T>>,
            value: NonuniqueBox<Option<T>>,
            // May borrow from self.value, so must drop first.
            seq: ManuallyDrop<Box<dyn Seq + 'a>>,
        }

        impl<'a, T: 'a> Drop for BoxSeq<'a, T> {
            fn drop(&mut self) {
                unsafe { ManuallyDrop::drop(&mut self.seq) }
            }
        }

        impl<'a, T> Seq for BoxSeq<'a, T>
        where
            T: Deserialize,
        {
            fn element(&mut self) -> Result<&mut dyn Visitor> {
                self.seq.element()
            }

            fn finish(&mut self) -> Result<()> {
                self.seq.finish()?;
                *self.seq = Box::new(Ignore);
                *self.out = Some(Box::new(self.value.take().unwrap()));
                Ok(())
            }
        }

        struct BoxMap<'a, T: 'a> {
            out: &'a mut Option<Box<T>>,
            value: NonuniqueBox<Option<T>>,
            // May borrow from self.value, so must drop first.
            map: ManuallyDrop<Box<dyn Map + 'a>>,
        }

        impl<'a, T: 'a> Drop for BoxMap<'a, T> {
            fn drop(&mut self) {
                unsafe { ManuallyDrop::drop(&mut self.map) }
            }
        }

        impl<'a, T> Map for BoxMap<'a, T>
        where
            T: Deserialize,
        {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor> {
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

impl<T> Deserialize for Option<T>
where
    T: Deserialize,
{
    #[inline]
    fn default() -> Option<Self> {
        Some(None)
    }

    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        make_place!(Place);

        impl<T> Visitor for Place<Option<T>>
        where
            T: Deserialize,
        {
            fn null(&mut self) -> Result<()> {
                self.out = Some(None);
                Ok(())
            }

            fn boolean(&mut self, b: bool) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).boolean(b)
            }

            fn string(&mut self, s: &str) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).string(s)
            }

            fn negative(&mut self, n: i64) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).negative(n)
            }

            fn nonnegative(&mut self, n: u64) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).nonnegative(n)
            }

            fn float(&mut self, n: f64) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).float(n)
            }

            fn seq(&mut self) -> Result<Box<dyn Seq + '_>> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).seq()
            }

            fn map(&mut self) -> Result<Box<dyn Map + '_>> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).map()
            }
        }

        Place::new(out)
    }
}

impl<A, B> Deserialize for (A, B)
where
    A: Deserialize,
    B: Deserialize,
{
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        make_place!(Place);

        impl<A, B> Visitor for Place<(A, B)>
        where
            A: Deserialize,
            B: Deserialize,
        {
            fn seq(&mut self) -> Result<Box<dyn Seq + '_>> {
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

        impl<'a, A, B> Seq for TupleBuilder<'a, A, B>
        where
            A: Deserialize,
            B: Deserialize,
        {
            fn element(&mut self) -> Result<&mut dyn Visitor> {
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

impl<T> Deserialize for Vec<T>
where
    T: Deserialize,
{
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        make_place!(Place);

        impl<T> Visitor for Place<Vec<T>>
        where
            T: Deserialize,
        {
            fn seq(&mut self) -> Result<Box<dyn Seq + '_>> {
                Ok(Box::new(VecBuilder {
                    out: &mut self.out,
                    vec: Vec::new(),
                    element: None,
                }))
            }
        }

        struct VecBuilder<'a, T: 'a> {
            out: &'a mut Option<Vec<T>>,
            vec: Vec<T>,
            element: Option<T>,
        }

        impl<'a, T> VecBuilder<'a, T> {
            fn shift(&mut self) {
                if let Some(e) = self.element.take() {
                    self.vec.push(e);
                }
            }
        }

        impl<'a, T> Seq for VecBuilder<'a, T>
        where
            T: Deserialize,
        {
            fn element(&mut self) -> Result<&mut dyn Visitor> {
                self.shift();
                Ok(Deserialize::begin(&mut self.element))
            }

            fn finish(&mut self) -> Result<()> {
                self.shift();
                *self.out = Some(mem::take(&mut self.vec));
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl<T, const N: usize> Deserialize for [T; N]
where
    T: Deserialize,
{
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        make_place!(Place);

        impl<T, const N: usize> Visitor for Place<[T; N]>
        where
            T: Deserialize,
        {
            fn seq(&mut self) -> Result<Box<dyn Seq + '_>> {
                Ok(Box::new(ArrayBuilder {
                    out: &mut self.out,
                    array: unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() },
                    len: 0,
                    element: None,
                }))
            }
        }

        struct ArrayBuilder<'a, T: 'a, const N: usize> {
            out: &'a mut Option<[T; N]>,
            array: [MaybeUninit<T>; N],
            len: usize,
            element: Option<T>,
        }

        impl<'a, T, const N: usize> ArrayBuilder<'a, T, N> {
            fn shift(&mut self) -> Result<()> {
                if let Some(e) = self.element.take() {
                    self.array.get_mut(self.len).ok_or(Error)?.write(e);
                    self.len += 1;
                }
                Ok(())
            }
        }

        impl<'a, T, const N: usize> Seq for ArrayBuilder<'a, T, N>
        where
            T: Deserialize,
        {
            fn element(&mut self) -> Result<&mut dyn Visitor> {
                self.shift()?;
                Ok(Deserialize::begin(&mut self.element))
            }

            fn finish(&mut self) -> Result<()> {
                self.shift()?;
                if self.len < N {
                    return Err(Error);
                }
                // First drop any array that is already in the Place. This way
                // we can atomically move self.array into it and reset self.len
                // to 0, without the possibility of a panic between those two
                // steps.
                *self.out = None;
                *self.out = Some(unsafe { ptr::addr_of_mut!(self.array).cast::<[T; N]>().read() });
                self.len = 0;
                Ok(())
            }
        }

        impl<'a, T: 'a, const N: usize> Drop for ArrayBuilder<'a, T, N> {
            fn drop(&mut self) {
                for element in &mut self.array[..self.len] {
                    unsafe { ptr::drop_in_place(element.assume_init_mut()) };
                }
            }
        }

        Place::new(out)
    }
}

#[cfg(feature = "std")]
impl<K, V, H> Deserialize for HashMap<K, V, H>
where
    K: FromStr + Hash + Eq,
    V: Deserialize,
    H: BuildHasher + Default,
{
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        make_place!(Place);

        impl<K, V, H> Visitor for Place<HashMap<K, V, H>>
        where
            K: FromStr + Hash + Eq,
            V: Deserialize,
            H: BuildHasher + Default,
        {
            fn map(&mut self) -> Result<Box<dyn Map + '_>> {
                Ok(Box::new(MapBuilder {
                    out: &mut self.out,
                    map: HashMap::with_hasher(H::default()),
                    key: None,
                    value: None,
                }))
            }
        }

        struct MapBuilder<'a, K: 'a, V: 'a, H: 'a> {
            out: &'a mut Option<HashMap<K, V, H>>,
            map: HashMap<K, V, H>,
            key: Option<K>,
            value: Option<V>,
        }

        impl<'a, K, V, H> MapBuilder<'a, K, V, H>
        where
            K: Hash + Eq,
            H: BuildHasher,
        {
            fn shift(&mut self) {
                if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                    self.map.insert(k, v);
                }
            }
        }

        impl<'a, K, V, H> Map for MapBuilder<'a, K, V, H>
        where
            K: FromStr + Hash + Eq,
            V: Deserialize,
            H: BuildHasher + Default,
        {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor> {
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

impl<K, V> Deserialize for BTreeMap<K, V>
where
    K: FromStr + Ord,
    V: Deserialize,
{
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        make_place!(Place);

        impl<K, V> Visitor for Place<BTreeMap<K, V>>
        where
            K: FromStr + Ord,
            V: Deserialize,
        {
            fn map(&mut self) -> Result<Box<dyn Map + '_>> {
                Ok(Box::new(MapBuilder {
                    out: &mut self.out,
                    map: BTreeMap::new(),
                    key: None,
                    value: None,
                }))
            }
        }

        struct MapBuilder<'a, K: 'a, V: 'a> {
            out: &'a mut Option<BTreeMap<K, V>>,
            map: BTreeMap<K, V>,
            key: Option<K>,
            value: Option<V>,
        }

        impl<'a, K, V> MapBuilder<'a, K, V>
        where
            K: Ord,
        {
            fn shift(&mut self) {
                if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                    self.map.insert(k, v);
                }
            }
        }

        impl<'a, K, V> Map for MapBuilder<'a, K, V>
        where
            K: FromStr + Ord,
            V: Deserialize,
        {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor> {
                self.shift();
                self.key = Some(match K::from_str(k) {
                    Ok(key) => key,
                    Err(_) => return Err(Error),
                });
                Ok(Deserialize::begin(&mut self.value))
            }

            fn finish(&mut self) -> Result<()> {
                self.shift();
                *self.out = Some(mem::take(&mut self.map));
                Ok(())
            }
        }

        Place::new(out)
    }
}
