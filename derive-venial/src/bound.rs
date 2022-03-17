use proc_macro2::{Punct, Spacing, TokenStream};
use quote::TokenStreamExt;
use venial::{
    Enum, GenericBound, GenericParam, GenericParams, Struct, WhereClause, WhereClauseItem,
};

// --- TODO - add to venial ---

fn get_type_params(generics: &Option<GenericParams>) -> Vec<GenericParam> {
    let generics = if let Some(generics) = generics.as_ref() {
        generics
    } else {
        return Vec::new();
    };
    generics
        .params
        .inner
        .iter()
        .map(|(param, _punct)| param)
        .filter(|param| param._prefix.is_none())
        .cloned()
        .collect()
}

pub fn create_derive_where_clause(
    generics: &Option<GenericParams>,
    base_where_clause: &Option<WhereClause>,
    derived_trait: TokenStream,
) -> WhereClause {
    let mut where_clause = base_where_clause.clone().unwrap_or_default();

    for param in get_type_params(generics) {
        let item = WhereClauseItem {
            left_side: vec![param.name.clone().into()],
            bound: GenericBound {
                _colon: Punct::new(':', Spacing::Alone),
                tokens: derived_trait.clone().into_iter().collect(),
            },
        };

        where_clause = where_clause.with_item(item);
    }

    where_clause
}

pub struct InlineGenericArgs<'a>(&'a GenericParams);

#[allow(unused)]
pub fn as_inline_args(generics: &GenericParams) -> InlineGenericArgs<'_> {
    InlineGenericArgs(generics)
}

pub fn get_inline_generic_args_struct(struct_decl: &Struct) -> Option<InlineGenericArgs<'_>> {
    Some(InlineGenericArgs(struct_decl.generic_params.as_ref()?))
}

#[allow(unused)]
pub fn get_inline_generic_args_enum(enum_decl: &Enum) -> Option<InlineGenericArgs<'_>> {
    Some(InlineGenericArgs(enum_decl.generic_params.as_ref()?))
}

impl quote::ToTokens for InlineGenericArgs<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Punct::new('<', Spacing::Alone));

        for param in &self.0.params.inner {
            if param.0.is_lifetime() {
                param.0._prefix.to_tokens(tokens);
            }
            tokens.append(param.0.name.clone());
            tokens.append(Punct::new(',', Spacing::Alone));
        }

        tokens.append(Punct::new('>', Spacing::Alone));
    }
}
