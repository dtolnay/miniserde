use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_quote, Data, DataStruct, DeriveInput, Fields, Ident};
use crate::{attr, bound};

pub fn derive(input: DeriveInput) -> TokenStream {
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields,
        _ => panic!("currently only structs with named fields are supported"),
    };

    let ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let dummy = Ident::new(
        &format!("_IMPL_MINISERIALIZE_FOR_{}", ident),
        Span::call_site(),
    );

    let fieldname = &fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let fieldstr = fields.named.iter().map(attr::name_of_field);
    let index = 0usize..;

    let wrapper_generics = bound::with_lifetime_bound(&input.generics, "'__a");
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();
    let bound = parse_quote!(miniserde::Serialize);
    let bounded_where_clause = bound::where_clause_with_bound(&input.generics, bound);

    TokenStream::from(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy: () = {
            impl #impl_generics miniserde::Serialize for #ident #ty_generics #bounded_where_clause {
                fn begin(&self) -> miniserde::ser::Fragment {
                    miniserde::ser::Fragment::Map(miniserde::export::Box::new(__Map {
                        data: self,
                        state: 0,
                    }))
                }
            }

            struct __Map #wrapper_impl_generics #where_clause {
                data: &'__a #ident #ty_generics,
                state: miniserde::export::usize,
            }

            impl #wrapper_impl_generics miniserde::ser::Map for __Map #wrapper_ty_generics #bounded_where_clause {
                fn next(&mut self) -> miniserde::export::Option<(miniserde::export::Cow<miniserde::export::str>, &miniserde::Serialize)> {
                    let __state = self.state;
                    self.state = __state + 1;
                    match __state {
                        #(
                            #index => miniserde::export::Some((
                                miniserde::export::Cow::Borrowed(#fieldstr),
                                &self.data.#fieldname,
                            )),
                        )*
                        _ => miniserde::export::None,
                    }
                }
            }
        };
    })
}
