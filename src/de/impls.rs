#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use crate::lib::hash::{BuildHasher, Hash};
use crate::lib::mem;
use crate::lib::str::FromStr;
use crate::lib::*;

use crate::de::{Deserialize, Map, Seq, Visitor};
use crate::error::{Error, Result};
use crate::Place;

impl Deserialize for () {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
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
                impl Visitor for Place<$ty> {
                    fn negative(&mut self, n: i64) -> Result<()> {
                        if n >= $ty::min_value() as i64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(Error)
                        }
                    }

                    fn nonnegative(&mut self, n: u64) -> Result<()> {
                        if n <= $ty::max_value() as u64 {
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
                impl Visitor for Place<$ty> {
                    fn nonnegative(&mut self, n: u64) -> Result<()> {
                        if n <= $ty::max_value() as u64 {
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

impl<T: Deserialize> Deserialize for Box<T> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl<T: Deserialize> Visitor for Place<Box<T>> {
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
                let mut value = Box::new(None);
                let ptr = careful!(&mut *value as &mut Option<T>);
                Ok(Box::new(BoxSeq {
                    out: &mut self.out,
                    value,
                    seq: Deserialize::begin(ptr).seq()?,
                }))
            }

            fn map(&mut self) -> Result<Box<dyn Map + '_>> {
                let mut value = Box::new(None);
                let ptr = careful!(&mut *value as &mut Option<T>);
                Ok(Box::new(BoxMap {
                    out: &mut self.out,
                    value,
                    map: Deserialize::begin(ptr).map()?,
                }))
            }
        }

        struct BoxSeq<'a, T: 'a> {
            out: &'a mut Option<Box<T>>,
            value: Box<Option<T>>,
            seq: Box<dyn Seq + 'a>,
        }

        impl<'a, T: Deserialize> Seq for BoxSeq<'a, T> {
            fn element(&mut self) -> Result<&mut dyn Visitor> {
                self.seq.element()
            }

            fn finish(&mut self) -> Result<()> {
                self.seq.finish()?;
                *self.out = Some(Box::new(self.value.take().unwrap()));
                Ok(())
            }
        }

        struct BoxMap<'a, T: 'a> {
            out: &'a mut Option<Box<T>>,
            value: Box<Option<T>>,
            map: Box<dyn Map + 'a>,
        }

        impl<'a, T: Deserialize> Map for BoxMap<'a, T> {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor> {
                self.map.key(k)
            }

            fn finish(&mut self) -> Result<()> {
                self.map.finish()?;
                *self.out = Some(Box::new(self.value.take().unwrap()));
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    #[inline]
    fn default() -> Option<Self> {
        Some(None)
    }
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl<T: Deserialize> Visitor for Place<Option<T>> {
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

impl<A: Deserialize, B: Deserialize> Deserialize for (A, B) {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl<A: Deserialize, B: Deserialize> Visitor for Place<(A, B)> {
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

        impl<'a, A: Deserialize, B: Deserialize> Seq for TupleBuilder<'a, A, B> {
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

impl<T: Deserialize> Deserialize for Vec<T> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl<T: Deserialize> Visitor for Place<Vec<T>> {
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

        impl<'a, T: Deserialize> Seq for VecBuilder<'a, T> {
            fn element(&mut self) -> Result<&mut dyn Visitor> {
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
impl<K, V, H> Deserialize for HashMap<K, V, H>
where
    K: FromStr + Hash + Eq,
    V: Deserialize,
    H: BuildHasher + Default,
{
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
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

        impl<'a, K: Hash + Eq, V, H: BuildHasher> MapBuilder<'a, K, V, H> {
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

impl<K: FromStr + Ord, V: Deserialize> Deserialize for BTreeMap<K, V> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl<K: FromStr + Ord, V: Deserialize> Visitor for Place<BTreeMap<K, V>> {
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

        impl<'a, K: Ord, V> MapBuilder<'a, K, V> {
            fn shift(&mut self) {
                if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                    self.map.insert(k, v);
                }
            }
        }

        impl<'a, K: FromStr + Ord, V: Deserialize> Map for MapBuilder<'a, K, V> {
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
                *self.out = Some(mem::replace(&mut self.map, BTreeMap::new()));
                Ok(())
            }
        }

        Place::new(out)
    }
}
