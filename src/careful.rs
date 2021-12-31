// Cast away lifetimes. This is required because without recursion all state
// needs to be held on the heap and that heap data structure ends up being self
// referential. The serialization and deserialization logic manipulates frames
// on the heap in a way that ensures all internal references are live at the
// right times.
//
// The unsafety is contained to the implementation of miniserde::json and not
// exposed to Serialize and Deserialize impls, so the miniserde public API
// remains entirely safe to use.
//
//     unsafe { extend_lifetime!(EXPR as TYPE) }
//
// expands to:
//
//     std::mem::transmute::<TYPE, TYPE>(EXPR)
macro_rules! extend_lifetime {
    ($($cast:tt)*) => {
        extend_lifetime_impl!(() $($cast)*)
    };
}

macro_rules! extend_lifetime_impl {
    (($($expr:tt)*) as $t:ty) => {{
        let expr = $($expr)*;
        core::mem::transmute::<$t, $t>(expr)
    }};
    (($($expr:tt)*) $next:tt $($rest:tt)*) => {
        extend_lifetime_impl!(($($expr)* $next) $($rest)*)
    };
}
