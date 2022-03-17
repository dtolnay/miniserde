use crate::{
    attr,
    bound::{create_derive_where_clause, get_inline_generic_args_struct},
};

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use venial::{parse_declaration, Declaration, Enum, GenericParam, Struct, StructFields};
#[allow(unused)]
use venial::{
    Attribute, EnumDiscriminant, EnumVariant, GenericParams, NamedField, TupleField, TyExpr,
    VisMarker, WhereClause,
};

type MyError = ();

pub fn derive(input: TokenStream) -> Result<TokenStream, MyError> {
    let type_decl = parse_declaration(input);

    let res = match &type_decl {
        Declaration::Struct(struct_decl) => derive_struct(struct_decl)?,
        Declaration::Enum(enum_decl) => derive_enum(enum_decl)?,
        _ => panic!("can't parse type"),
    };

    #[cfg(FALSE)]
    {
        return Ok(expander::Expander::new("globalmacro")
            .add_comment("This is generated code!".to_owned())
            .dry(false)
            .verbose(true)
            .write_to_out_dir(res.clone())
            .unwrap_or_else(|e| {
                eprintln!("Failed to write to file: {:?}", e);
                res
            }));
    }

    #[allow(unreachable_code)]
    Ok(res)
}

fn derive_struct(struct_decl: &Struct) -> Result<TokenStream, MyError> {
    let name_ident = &struct_decl.name;

    let dummy = Ident::new(
        &format!("_IMPL_MINIDESERIALIZE_FOR_{}", name_ident),
        Span::call_site(),
    );

    let impl_generics = &struct_decl.generic_params;
    let inline_generics = get_inline_generic_args_struct(&struct_decl);
    let bounded_where_clause = create_derive_where_clause(
        &struct_decl.generic_params,
        &struct_decl.where_clause,
        quote!(miniserde::Serialize),
    );

    let fields = match &struct_decl.fields {
        StructFields::Unit => panic!("can't parse unit struct"),
        StructFields::Tuple(_fields) => panic!("can't parse tuple struct"),
        StructFields::Named(fields) => fields,
    };
    let field_names = fields
        .fields
        .inner
        .iter()
        .map(|field| &field.0.name)
        .collect::<Vec<_>>();
    let field_types = fields.fields.inner.iter().map(|field| &field.0.ty);
    let field_strings: Vec<_> = fields
        .fields
        .inner
        .iter()
        .map(|field| attr::name_of_field(&field.0))
        .collect::<Result<Vec<_>, ()>>()?;

    let wrapper_decl = struct_decl
        .clone()
        .with_param(GenericParam::lifetime("__a"));
    let wrapper_impl_generics = &wrapper_decl.generic_params;
    let wrapper_inline_generics = get_inline_generic_args_struct(&wrapper_decl);
    let wrapper_where_clause = struct_decl.where_clause.clone();

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy: () = {
            #[repr(C)]
            struct __Visitor #impl_generics #wrapper_where_clause {
                __out: miniserde::__private::Option<#name_ident #inline_generics>,
            }

            impl #impl_generics miniserde::Deserialize for #name_ident #inline_generics #bounded_where_clause {
                fn begin(__out: &mut miniserde::__private::Option<Self>) -> &mut dyn miniserde::de::Visitor {
                    unsafe {
                        &mut *{
                            __out
                            as *mut miniserde::__private::Option<Self>
                            as *mut __Visitor #inline_generics
                        }
                    }
                }
            }

            impl #impl_generics miniserde::de::Visitor for __Visitor #inline_generics #bounded_where_clause {
                fn map(&mut self) -> miniserde::Result<miniserde::__private::Box<dyn miniserde::de::Map + '_>> {
                    Ok(miniserde::__private::Box::new(__State {
                        #(
                            #field_names: miniserde::Deserialize::default(),
                        )*
                        __out: &mut self.__out,
                    }))
                }
            }

            struct __State #wrapper_impl_generics #wrapper_where_clause {
                #(
                    #field_names: miniserde::__private::Option<#field_types>,
                )*
                __out: &'__a mut miniserde::__private::Option<#name_ident #inline_generics>,
            }

            impl #wrapper_impl_generics miniserde::de::Map for __State #wrapper_inline_generics #bounded_where_clause {
                fn key(&mut self, __k: &miniserde::__private::str) -> miniserde::Result<&mut dyn miniserde::de::Visitor> {
                    match __k {
                        #(
                            #field_strings => miniserde::__private::Ok(miniserde::Deserialize::begin(&mut self.#field_names)),
                        )*
                        _ => miniserde::__private::Ok(<dyn miniserde::de::Visitor>::ignore()),
                    }
                }

                fn finish(&mut self) -> miniserde::Result<()> {
                    #(
                        let #field_names = self.#field_names.take().ok_or(miniserde::Error)?;
                    )*
                    *self.__out = miniserde::__private::Some(#name_ident {
                        #(
                            #field_names,
                        )*
                    });
                    miniserde::__private::Ok(())
                }
            }
        };
    })
}

fn derive_enum(enum_decl: &Enum) -> Result<TokenStream, MyError> {
    if enum_decl.generic_params.is_some() {
        panic!("Enums with generics are not supported");
    }

    let name_ident = &enum_decl.name;
    let dummy = Ident::new(
        &format!("_IMPL_MINIDESERIALIZE_FOR_{}", name_ident),
        Span::call_site(),
    );

    let variant_idents = enum_decl
        .variants
        .inner
        .iter()
        .map(|variant| match variant.0.contents {
            StructFields::Unit => Ok(&variant.0.name),
            _ => panic!("Invalid variant: only simple enum variants without fields are supported"),
        })
        .collect::<Result<Vec<_>, MyError>>()?;
    let variant_names = enum_decl
        .variants
        .inner
        .iter()
        .map(|variant| attr::name_of_variant(&variant.0))
        .collect::<Result<Vec<_>, MyError>>()?;

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy: () = {
            #[repr(C)]
            struct __Visitor {
                __out: miniserde::__private::Option<#name_ident>,
            }

            impl miniserde::Deserialize for #name_ident {
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
                        #( #variant_names => #name_ident::#variant_idents, )*
                        _ => return miniserde::__private::Err(miniserde::Error),
                    };
                    self.__out = miniserde::__private::Some(value);
                    miniserde::__private::Ok(())
                }
            }
        };
    })
}
