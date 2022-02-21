use crate::{attr, bound};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, FieldsNamed, Ident, Result,
};

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => derive_struct(&input, fields),
        Data::Enum(enumeration) => derive_enum(&input, enumeration),
        Data::Struct(_) => Err(Error::new(
            Span::call_site(),
            "currently only structs with named fields are supported",
        )),
        Data::Union(_) => Err(Error::new(
            Span::call_site(),
            "currently only structs and enums are supported by this derive",
        )),
    }
}

pub fn derive_struct(input: &DeriveInput, fields: &FieldsNamed) -> Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let dummy = Ident::new(
        &format!("_IMPL_MINIDESERIALIZE_FOR_{}", ident),
        Span::call_site(),
    );

    let fieldname = fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let fieldty = fields.named.iter().map(|f| &f.ty);
    let fieldstr = fields
        .named
        .iter()
        .map(attr::name_of_field)
        .collect::<Result<Vec<_>>>()?;

    let wrapper_generics = bound::with_lifetime_bound(&input.generics, "'__a");
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();
    let bound = parse_quote!(miniserde::Deserialize);
    let bounded_where_clause = bound::where_clause_with_bound(&input.generics, bound);

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy: () = {
            #[repr(C)]
            struct __Visitor #impl_generics #where_clause {
                __out: miniserde::__private::Option<#ident #ty_generics>,
            }

            impl #impl_generics miniserde::Deserialize for #ident #ty_generics #bounded_where_clause {
                fn begin(__out: &mut miniserde::__private::Option<Self>) -> &mut dyn miniserde::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut miniserde::__private::Option<Self>
                            as *mut __Visitor #ty_generics
                        }
                    }
                }
            }

            impl #impl_generics miniserde::de::Visitor for __Visitor #ty_generics #bounded_where_clause {
                fn map(&mut self) -> miniserde::Result<miniserde::__private::Box<dyn miniserde::de::Map + '_>> {
                    Ok(miniserde::__private::Box::new(__State {
                        #(
                            #fieldname: miniserde::Deserialize::default(),
                        )*
                        __out: &mut self.__out,
                    }))
                }
            }

            struct __State #wrapper_impl_generics #where_clause {
                #(
                    #fieldname: miniserde::__private::Option<#fieldty>,
                )*
                __out: &'__a mut miniserde::__private::Option<#ident #ty_generics>,
            }

            impl #wrapper_impl_generics miniserde::de::Map for __State #wrapper_ty_generics #bounded_where_clause {
                fn key(&mut self, __k: &miniserde::__private::str) -> miniserde::Result<&mut dyn miniserde::de::Visitor> {
                    match __k {
                        #(
                            #fieldstr => miniserde::__private::Ok(miniserde::Deserialize::begin(&mut self.#fieldname)),
                        )*
                        _ => miniserde::__private::Ok(<dyn miniserde::de::Visitor>::ignore()),
                    }
                }

                fn finish(&mut self) -> miniserde::Result<()> {
                    #(
                        let #fieldname = self.#fieldname.take().ok_or(miniserde::Error)?;
                    )*
                    *self.__out = miniserde::__private::Some(#ident {
                        #(
                            #fieldname,
                        )*
                    });
                    miniserde::__private::Ok(())
                }
            }
        };
    })
}

pub fn derive_enum(input: &DeriveInput, enumeration: &DataEnum) -> Result<TokenStream> {
    if input.generics.lt_token.is_some() || input.generics.where_clause.is_some() {
        return Err(Error::new(
            Span::call_site(),
            "Enums with generics are not supported",
        ));
    }

    let ident = &input.ident;
    let dummy = Ident::new(
        &format!("_IMPL_MINIDESERIALIZE_FOR_{}", ident),
        Span::call_site(),
    );

    let var_idents = enumeration
        .variants
        .iter()
        .map(|variant| match variant.fields {
            Fields::Unit => Ok(&variant.ident),
            _ => Err(Error::new_spanned(
                variant,
                "Invalid variant: only simple enum variants without fields are supported",
            )),
        })
        .collect::<Result<Vec<_>>>()?;
    let names = enumeration
        .variants
        .iter()
        .map(attr::name_of_variant)
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy: () = {
            #[repr(C)]
            struct __Visitor {
                __out: miniserde::__private::Option<#ident>,
            }

            impl miniserde::Deserialize for #ident {
                fn begin(__out: &mut miniserde::__private::Option<Self>) -> &mut dyn miniserde::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut miniserde::__private::Option<Self>
                            as *mut __Visitor
                        }
                    }
                }
            }

            impl miniserde::de::Visitor for __Visitor {
                fn string(&mut self, s: &miniserde::__private::str) -> miniserde::Result<()> {
                    let value = match s {
                        #( #names => #ident::#var_idents, )*
                        _ => return miniserde::__private::Err(miniserde::Error),
                    };
                    self.__out = miniserde::__private::Some(value);
                    miniserde::__private::Ok(())
                }
            }
        };
    })
}
