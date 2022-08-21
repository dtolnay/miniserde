use crate::ser::{Fragment, Serialize};

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
