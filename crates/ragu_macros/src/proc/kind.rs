use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Result, Token, Type,
    parse::{Parse, ParseStream},
};

use crate::{path_resolution::RaguCorePath, substitution::replace_inferences};

pub struct Input {
    f: Type,
    _semicolon: Token![;],
    cast: Option<Token![@]>,
    path: Type,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            f: input.parse()?,
            _semicolon: input.parse()?,
            cast: if input.peek(Token![@]) {
                Some(input.parse()?)
            } else {
                None
            },
            path: input.parse()?,
        })
    }
}

pub fn evaluate(input: Input, ragu_core_path: RaguCorePath) -> syn::Result<TokenStream> {
    let Input { f, path, cast, .. } = input;

    let mut subst = path.clone();
    replace_inferences(&mut subst, &f);

    if cast.is_none() {
        Ok(
            quote!(<#subst as #ragu_core_path::gadgets::Gadget<'static, ::core::marker::PhantomData<#f>>>::Kind),
        )
    } else {
        Ok(quote!(#subst))
    }
}

#[rustfmt::skip]
#[test]
fn test_evaluate() {
    use syn::parse_quote;

    assert_eq!(
        evaluate(
            parse_quote!(F; MyGadget<'_, _, C, 5>),
            RaguCorePath::default()
        )
        .unwrap()
        .to_string(),
        quote!(
            <MyGadget<'static, ::core::marker::PhantomData<F>, C, 5> as ::ragu_core::gadgets::Gadget<'static, ::core::marker::PhantomData<F>>>::Kind
        )
        .to_string()
    );
}

#[rustfmt::skip]
#[test]
fn test_extra() {
    use syn::parse_quote;

    assert_eq!(
        evaluate(
            parse_quote!(F; @EndoscalingOutput<'_, _, C>),
            RaguCorePath::default()
        )
        .unwrap()
        .to_string(),
        quote!(
            EndoscalingOutput<'static, ::core::marker::PhantomData<F>, C>
        )
        .to_string()
    );
}
