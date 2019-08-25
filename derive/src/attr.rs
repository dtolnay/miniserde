use syn::{Attribute, Lit, Meta, Field, NestedMeta, Variant};

/// Find the value of a #[serde(rename = "xxx")] attribute.
fn find_rename_attr(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        let segments = &attr.path.segments;
        if !(segments.len() == 1 && segments[0].ident == "serde") {
            continue;
        }

        let list = match attr.parse_meta() {
            Ok(Meta::List(list)) => list,
            _ => panic!("unsupported attribute"),
        };

        for meta in list.nested {
            if let NestedMeta::Meta(Meta::NameValue(value)) = meta {
                if value.path.is_ident("rename") {
                    if let Lit::Str(s) = value.lit {
                        return Some(s.value());
                    }
                }
            }
            panic!("unsupported attribute");
        }
    }
    None
}

/// Determine the name of a field, respecting a rename attribute.
pub fn name_of_field(field: &Field) -> String {
    find_rename_attr(&field.attrs)
        .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string())
}

/// Determine the name of a variant, respecting a rename attribute.
pub fn name_of_variant(var: &Variant) -> String {
    find_rename_attr(&var.attrs)
        .unwrap_or_else(|| var.ident.to_string())
}
