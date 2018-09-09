use syn::{Field, Lit, Meta, NestedMeta};

pub fn name_of_field(field: &Field) -> String {
    let mut rename = None;

    for attr in &field.attrs {
        let segments = &attr.path.segments;
        if !(segments.len() == 1 && segments[0].ident == "serde") {
            continue;
        }

        let list = match attr.interpret_meta() {
            Some(Meta::List(list)) => list,
            _ => panic!("unsupported attribute"),
        };

        for meta in list.nested {
            if let NestedMeta::Meta(Meta::NameValue(value)) = meta {
                if value.ident == "rename" && rename.is_none() {
                    if let Lit::Str(s) = value.lit {
                        rename = Some(s.value());
                        continue;
                    }
                }
            }
            panic!("unsupported attribute");
        }
    }

    rename.unwrap_or_else(|| field.ident.as_ref().unwrap().to_string())
}
