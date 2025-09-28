use crate::{attr, bound, fallback, private};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse_quote, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, FieldsNamed, Result,
};

pub fn derive(input: &DeriveInput) -> TokenStream {
    match try_expand(input) {
        Ok(expanded) => expanded,
        // If there are invalid attributes in the input, expand to a Serialize
        // impl anyway to minimize spurious secondary errors in other code that
        // serializes this type.
        Err(error) => fallback::ser(input, error),
    }
}

fn try_expand(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => derive_struct(input, fields),
        Data::Enum(enumeration) => derive_enum(input, enumeration),
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

fn derive_struct(input: &DeriveInput, fields: &FieldsNamed) -> Result<TokenStream> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fieldname = &fields.named.iter().map(|f| &f.ident).collect::<Vec<_>>();
    let fieldstr = fields
        .named
        .iter()
        .map(attr::name_of_field)
        .collect::<Result<Vec<_>>>()?;
    let index = 0usize..;

    let wrapper_generics = bound::with_lifetime_bound(&input.generics, "'__a");
    let (wrapper_impl_generics, wrapper_ty_generics, _) = wrapper_generics.split_for_impl();
    let bound = parse_quote!(miniserde::Serialize);
    let bounded_where_clause = bound::where_clause_with_bound(&input.generics, bound);
    let private2 = private;

    Ok(quote! {
        #[allow(deprecated, non_upper_case_globals)]
        const _: () = {
            impl #impl_generics miniserde::Serialize for #ident #ty_generics #bounded_where_clause {
                fn begin(&self) -> miniserde::ser::Fragment {
                    miniserde::ser::Fragment::Map(miniserde::#private::Box::new(__Map {
                        data: self,
                        state: 0,
                    }))
                }
            }

            struct __Map #wrapper_impl_generics #where_clause {
                data: &'__a #ident #ty_generics,
                state: miniserde::#private::usize,
            }

            impl #wrapper_impl_generics miniserde::ser::Map for __Map #wrapper_ty_generics #bounded_where_clause {
                fn next(&mut self) -> miniserde::#private::Option<(miniserde::#private::Cow<miniserde::#private::str>, &dyn miniserde::Serialize)> {
                    let __state = self.state;
                    self.state = __state + 1;
                    match __state {
                        #(
                            #index => miniserde::#private2::Some((
                                miniserde::#private2::Cow::Borrowed(#fieldstr),
                                &self.data.#fieldname,
                            )),
                        )*
                        _ => miniserde::#private::None,
                    }
                }
            }
        };
    })
}

fn derive_enum(input: &DeriveInput, enumeration: &DataEnum) -> Result<TokenStream> {
    if input.generics.lt_token.is_some() || input.generics.where_clause.is_some() {
        return Err(Error::new(
            Span::call_site(),
            "Enums with generics are not supported",
        ));
    }

    let ident = &input.ident;

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
    let private2 = private;

    Ok(quote! {
        #[allow(deprecated, non_upper_case_globals)]
        const _: () = {
            impl miniserde::Serialize for #ident {
                fn begin(&self) -> miniserde::ser::Fragment {
                    match self {
                        #(
                            #ident::#var_idents => {
                                miniserde::ser::Fragment::Str(miniserde::#private2::Cow::Borrowed(#names))
                            }
                        )*
                    }
                }
            }
        };
    })
}
