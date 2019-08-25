use proc_macro2::{Span, TokenStream};
use syn::punctuated::Punctuated;
use syn::{
    parse_quote, GenericParam, Generics, Lifetime, LifetimeDef, TypeParamBound, WhereClause,
    WherePredicate,
};

pub fn with_lifetime_bound(generics: &Generics, lifetime: &str) -> Generics {
    let bound = Lifetime::new(lifetime, Span::call_site());
    let def = LifetimeDef {
        attrs: Vec::new(),
        lifetime: bound.clone(),
        colon_token: None,
        bounds: Punctuated::new(),
    };

    let params = Some(GenericParam::Lifetime(def))
        .into_iter()
        .chain(generics.params.iter().cloned().map(|mut param| {
            match &mut param {
                GenericParam::Lifetime(param) => {
                    param.bounds.push(bound.clone());
                }
                GenericParam::Type(param) => {
                    param.bounds.push(TypeParamBound::Lifetime(bound.clone()));
                }
                GenericParam::Const(_) => {}
            }
            param
        }))
        .collect();

    Generics {
        params: params,
        ..generics.clone()
    }
}

pub fn where_clause_with_bound(generics: &Generics, bound: TokenStream) -> WhereClause {
    let new_predicates = generics.type_params().map::<WherePredicate, _>(|param| {
        let param = &param.ident;
        parse_quote!(#param : #bound)
    });

    let mut generics = generics.clone();
    generics
        .make_where_clause()
        .predicates
        .extend(new_predicates);
    generics.where_clause.unwrap()
}
