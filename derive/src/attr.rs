use syn::{Attribute, Error, Field, Lit, Meta, NestedMeta, Result, Variant};

/// Find the value of a #[serde(rename = "...")] attribute.
fn attr_rename(attrs: &[Attribute]) -> Result<Option<String>> {
    for attr in attrs {
        if !attr.path.is_ident("serde") {
            continue;
        }

        let list = match attr.parse_meta()? {
            Meta::List(list) => list,
            other => return Err(Error::new_spanned(other, "unsupported attribute")),
        };

        for meta in &list.nested {
            if let NestedMeta::Meta(Meta::NameValue(value)) = meta {
                if value.path.is_ident("rename") {
                    if let Lit::Str(s) = &value.lit {
                        return Ok(Some(s.value()));
                    }
                }
            }
            return Err(Error::new_spanned(meta, "unsupported attribute"));
        }
    }
    Ok(None)
}

/// Determine the name of a field, respecting a rename attribute.
pub fn name_of_field(field: &Field) -> Result<String> {
    let rename = attr_rename(&field.attrs)?;
    Ok(rename.unwrap_or_else(|| field.ident.as_ref().unwrap().to_string()))
}

/// Determine the name of a variant, respecting a rename attribute.
pub fn name_of_variant(var: &Variant) -> Result<String> {
    let rename = attr_rename(&var.attrs)?;
    Ok(rename.unwrap_or_else(|| var.ident.to_string()))
}
