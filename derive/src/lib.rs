#![allow(
    clippy::module_name_repetitions,
    clippy::needless_pass_by_value,
    clippy::unseparated_literal_suffix
)]

extern crate proc_macro;

mod attr;
mod bound;
mod de;
mod fallback;
mod ser;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{ToTokens, TokenStreamExt as _};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Serialize, attributes(serde))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    ser::derive(&input).into()
}

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    de::derive(&input).into()
}

#[allow(non_camel_case_types)]
struct private;

impl ToTokens for private {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(Ident::new(
            concat!("__private", env!("CARGO_PKG_VERSION_PATCH")),
            Span::call_site(),
        ));
    }
}
