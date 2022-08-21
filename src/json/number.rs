use crate::de::{Deserialize, Visitor};
use crate::error::Result;
use crate::ser::{Fragment, Serialize};
use crate::Place;

/// A JSON number represented by some Rust primitive.
#[derive(Clone, Debug)]
pub enum Number {
    U64(u64),
    I64(i64),
    F64(f64),
}

impl Serialize for Number {
    fn begin(&self) -> Fragment {
        match self {
            Number::U64(n) => Fragment::U64(*n),
            Number::I64(n) => Fragment::I64(*n),
            Number::F64(n) => Fragment::F64(*n),
        }
    }
}

impl Deserialize for Number {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl Visitor for Place<Number> {
            fn negative(&mut self, n: i64) -> Result<()> {
                self.out = Some(Number::I64(n));
                Ok(())
            }

            fn nonnegative(&mut self, n: u64) -> Result<()> {
                self.out = Some(Number::U64(n));
                Ok(())
            }

            fn float(&mut self, n: f64) -> Result<()> {
                self.out = Some(Number::F64(n));
                Ok(())
            }
        }

        Place::new(out)
    }
}
