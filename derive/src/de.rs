use crate::{attr, bound};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_quote, Data, DataStruct, DataEnum, DeriveInput, Fields, FieldsNamed, Ident};

pub fn derive(input: DeriveInput) -> TokenStream {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => derive_struct(&input, fields),
        Data::Enum(ref _enum) => derive_enum(&input, _enum),
        _ => panic!("currently only structs with named fields are supported"),
    }
}

pub fn derive_struct(input: &DeriveInput, fields: &FieldsNamed) -> TokenStream {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let dummy = Ident::new(
        &format!("_IMPL_MINIDESERIALIZE_FOR_{}", ident),
        Span::call_site(),
    );

    let fieldname = &fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let fieldname2 = fieldname;
    let fieldty = fields.named.iter().map(|f| &f.ty);
    let fieldstr = fields.named.iter().map(attr::name_of_field);

    let wrapper_generics = bound::with_lifetime_bound(&input.generics, "'__a");
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();
    let bound = parse_quote!(miniserde::Deserialize);
    let bounded_where_clause = bound::where_clause_with_bound(&input.generics, bound);

    TokenStream::from(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy: () = {
            #[repr(C)]
            struct __Visitor #impl_generics #where_clause {
                __out: miniserde::export::Option<#ident #ty_generics>,
            }

            impl #impl_generics miniserde::Deserialize for #ident #ty_generics #bounded_where_clause {
                fn begin(__out: &mut miniserde::export::Option<Self>) -> &mut dyn miniserde::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut miniserde::export::Option<Self>
                            as *mut __Visitor #ty_generics
                        }
                    }
                }
            }

            impl #impl_generics miniserde::de::Visitor for __Visitor #ty_generics #bounded_where_clause {
                fn map(&mut self) -> miniserde::Result<miniserde::export::Box<dyn miniserde::de::Map + '_>> {
                    Ok(miniserde::export::Box::new(__State {
                        #(
                            #fieldname: miniserde::Deserialize::default(),
                        )*
                        __out: &mut self.__out,
                    }))
                }
            }

            struct __State #wrapper_impl_generics #where_clause {
                #(
                    #fieldname: miniserde::export::Option<#fieldty>,
                )*
                __out: &'__a mut miniserde::export::Option<#ident #ty_generics>,
            }

            impl #wrapper_impl_generics miniserde::de::Map for __State #wrapper_ty_generics #bounded_where_clause {
                fn key(&mut self, __k: &miniserde::export::str) -> miniserde::Result<&mut dyn miniserde::de::Visitor> {
                    match __k {
                        #(
                            #fieldstr => miniserde::export::Ok(miniserde::Deserialize::begin(&mut self.#fieldname2)),
                        )*
                        _ => miniserde::export::Ok(miniserde::de::Visitor::ignore()),
                    }
                }

                fn finish(&mut self) -> miniserde::Result<()> {
                    #(
                        let #fieldname = self.#fieldname2.take().ok_or(miniserde::Error)?;
                    )*
                    *self.__out = miniserde::export::Some(#ident {
                        #(
                            #fieldname,
                        )*
                    });
                    miniserde::export::Ok(())
                }
            }
        };
    })
}

pub fn derive_enum(input: &DeriveInput, _enum: &DataEnum) -> TokenStream {
    if input.generics.lt_token.is_some() || input.generics.where_clause.is_some() {
        panic!("Enums with generics are not supported");
    }

    let ident = &input.ident;
    let dummy = Ident::new(
        &format!("_IMPL_MINIDESERIALIZE_FOR_{}", ident),
        Span::call_site(),
    );

    let var_idents = _enum.variants.iter().map(|variant| {
        match variant.fields {
            Fields::Unit => {},
            _ => panic!(
                "Invalid variant {}:  only simple enum variants without fields are supported", 
                variant.ident,
            ),
        }
        &variant.ident
    });
    let names = _enum.variants.iter().map(attr::name_of_variant);

    TokenStream::from(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy: () = {
            #[repr(C)]
            struct __Visitor {
                __out: miniserde::export::Option<#ident>,
            }

            impl miniserde::Deserialize for #ident {
                fn begin(__out: &mut miniserde::export::Option<Self>) -> &mut dyn miniserde::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut miniserde::export::Option<Self>
                            as *mut __Visitor
                        }
                    }
                }
            }

            impl miniserde::de::Visitor for __Visitor {
                fn string(&mut self, s: &str) -> miniserde::Result<()> {
                    let value = match s {
                        #( #names => #ident::#var_idents, )*
                        _ => { return Err(miniserde::Error) },
                    };
                    self.__out = Some(value);
                    Ok(())
                }
            }
        };
    })
}
