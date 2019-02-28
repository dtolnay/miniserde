#![recursion_limit = "128"]

extern crate proc_macro;

mod attr;
mod bound;
mod de;
mod ser;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Serialize, attributes(serde))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    ser::derive(parse_macro_input!(input as DeriveInput))
}

#[proc_macro_derive(Deserialize, attributes(serde))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    de::derive(parse_macro_input!(input as DeriveInput))
}
