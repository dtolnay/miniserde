/// Macro to define a "place" type compatible with deserialization.
///
/// [Refer to the `miniserde::de` documentation for examples.](de/index.html)
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
        struct $name<__T> {
            out: $crate::export::Option<__T>,
        }

        impl<__T> $name<__T> {
            fn new(out: &mut $crate::export::Option<__T>) -> &mut Self {
                type __From<'a, __T> = &'a mut $crate::export::Option<__T>;
                type __To<'a, __T> = &'a mut $name<__T>;
                unsafe { $crate::export::mem::transmute::<__From<__T>, __To<__T>>(out) }
            }
        }
    };
}
