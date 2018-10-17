/// Macro to define a "place" type compatible with deserialization.
///
/// [Refer to the `miniserde::de` documentation for examples.][::de]
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
            #[cfg_attr(feature = "cargo-clippy", allow(new_ret_no_self))]
            fn new(out: &mut $crate::export::Option<__T>) -> &mut Self {
                unsafe {
                    &mut *{
                        out
                        as *mut $crate::export::Option<__T>
                        as *mut $name<__T>
                    }
                }
            }
        }
    };
}
