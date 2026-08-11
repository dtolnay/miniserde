use crate::private;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Attribute, Field, LitStr, Path, Result, Variant};

/// Find the value of a #[serde(rename = "...")], #[serde(default)], and #[serde(default = "...")] attributes.
fn attr_parse(attrs: &[Attribute]) -> Result<(Option<String>, Option<TokenStream>)> {
    let mut rename = None;
    let mut default = None;

    for attr in attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                let s: LitStr = meta.value()?.parse()?;
                if rename.is_some() {
                    return Err(meta.error("duplicate rename attribute"));
                }
                rename = Some(s.value());
                Ok(())
            } else if meta.path.is_ident("default") {
                if default.is_some() {
                    return Err(meta.error("duplicate default attribute"));
                }

                if meta.input.is_empty() {
                    default = Some(
                        quote!(miniserde::#private::Some(::core::default::Default::default())),
                    );
                    return Ok(());
                }

                let lit: LitStr = meta.value()?.parse()?;
                let path: Path = lit.parse()?;
                default = Some(quote!(miniserde::#private::Some(#path())));
                Ok(())
            } else {
                Err(meta.error("unsupported attribute"))
            }
        })?;
    }

    Ok((rename, default))
}

/// Determine the name of a field, respecting a rename attribute.
pub fn name_of_field(field: &Field) -> Result<String> {
    let (rename, _) = attr_parse(&field.attrs)?;
    Ok(rename.unwrap_or_else(|| unraw(field.ident.as_ref().unwrap())))
}

/// Determine the name of a variant, respecting a rename attribute.
pub fn name_of_variant(var: &Variant) -> Result<String> {
    let (rename, _) = attr_parse(&var.attrs)?;
    Ok(rename.unwrap_or_else(|| unraw(&var.ident)))
}

/// Determine the default of a field, respecting the default attribute.
pub fn default_of_field(field: &Field) -> Result<TokenStream> {
    let (_, default) = attr_parse(&field.attrs)?;
    Ok(default.unwrap_or_else(|| quote!(miniserde::Deserialize::default())))
}

fn unraw(ident: &Ident) -> String {
    ident.to_string().trim_start_matches("r#").to_owned()
}
