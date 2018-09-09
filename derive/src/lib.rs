#![recursion_limit = "128"]

#[macro_use]
extern crate quote;

#[macro_use]
extern crate syn;

extern crate proc_macro;
extern crate proc_macro2;

mod attr;
mod bound;
mod de;
mod ser;

use proc_macro::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(MiniSerialize, attributes(serde))]
pub fn derive_mini_serialize(input: TokenStream) -> TokenStream {
    ser::derive(parse_macro_input!(input as DeriveInput))
}

#[proc_macro_derive(MiniDeserialize, attributes(serde))]
pub fn derive_mini_deserialize(input: TokenStream) -> TokenStream {
    de::derive(parse_macro_input!(input as DeriveInput))
}
