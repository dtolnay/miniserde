/// Macro to define a "place" type compatible with deserialization.
///
/// [Refer to the `miniserde::de` documentation for examples.][crate::de]
///
/// This macro expands to:
///
/// ```rust
/// # macro_rules! make_place {
/// #     ($name:ident) => {
/// struct $name<T> {
///     out: Option<T>,
/// }
///
/// impl<T> $name<T> {
///     fn new(out: &mut Option<T>) -> &mut Self {
///         /* ... */
/// #         unimplemented!()
///     }
/// }
/// #     };
/// # }
/// #
/// # make_place!(Place);
/// ```
#[macro_export]
macro_rules! make_place {
    ($name:ident) => {
        #[repr(C)]
        struct $name<__T, __E> {
            out: $crate::place::Cell<__T, __E>,
        }

        impl<__T, __E> $name<__T, __E>
        where
            __E: $crate::de::VisitorError,
        {
            fn new(out: &mut $crate::place::Cell<__T, __E>) -> &mut Self {
                unsafe { &mut *(out as *mut $crate::place::Cell<__T, __E> as *mut $name<__T, __E>) }
            }
        }
    };
}

pub enum Cell<T, E> {
    Ok(T),
    Err(E),
    Empty,
}

impl<T, E> Cell<T, E> {
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Cell::Empty)
    }

    pub fn set(&mut self, val: T) {
        if !self.is_err() {
            *self = Self::Ok(val);
        }
    }

    pub fn err(&mut self, err: E) {
        *self = Self::Err(err);
    }

    pub fn is_err(&self) -> bool {
        matches!(self, Cell::Err(_))
    }

    pub fn map<F, O>(self, f: F) -> Cell<O, E>
    where
        F: FnOnce(T) -> O,
    {
        match self {
            Cell::Ok(v) => Cell::Ok(f(v)),
            Cell::Err(e) => Cell::Err(e),
            Cell::Empty => Cell::Empty,
        }
    }
}
