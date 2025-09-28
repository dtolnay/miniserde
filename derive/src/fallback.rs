use crate::private;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub(crate) fn ser(input: &DeriveInput, error: syn::Error) -> TokenStream {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let error = error.into_compile_error();

    quote! {
        #error

        #[allow(deprecated)]
        impl #impl_generics miniserde::Serialize for #ident #ty_generics #where_clause {
            fn begin(&self) -> miniserde::ser::Fragment {
                miniserde::#private::unreachable!()
            }
        }
    }
}

pub(crate) fn de(input: &DeriveInput, error: syn::Error) -> TokenStream {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let error = error.into_compile_error();

    quote! {
        #error

        #[allow(deprecated)]
        impl #impl_generics miniserde::Deserialize for #ident #ty_generics #where_clause {
            fn begin(__out: &mut miniserde::#private::Option<Self>) -> &mut dyn miniserde::de::Visitor {
                miniserde::#private::unreachable!()
            }
        }
    }
}
