use proc_macro2::{Span, TokenStream, TokenTree};
use quote::format_ident;
use syn::{
    Attribute, Error, GenericArgument, GenericParam, Generics, Ident, Lifetime, Meta,
    PathArguments, Result, TypeParam, TypeParamBound, spanned::Spanned,
};

pub fn attr_is(attr: &Attribute, needle: &str) -> bool {
    if !attr.path().is_ident("ragu") {
        return false;
    }
    match &attr.meta {
        Meta::List(list) => list.tokens.clone().into_iter().any(|tt| match tt {
            TokenTree::Ident(ref ident) => ident == needle,
            _ => false,
        }),
        _ => false,
    }
}

#[cfg(test)]
use syn::parse_quote;

#[test]
fn test_attr_is() {
    let attr: Attribute = parse_quote!(#[ragu(driver)]);
    assert!(attr_is(&attr, "driver"));
    assert!(!attr_is(&attr, "not_driver"));

    let attr: Attribute = parse_quote!(#[ragu(not_driver)]);
    assert!(!attr_is(&attr, "driver"));
    assert!(attr_is(&attr, "not_driver"));

    let attr: Attribute = parse_quote!(#[ragu]);
    assert!(!attr_is(&attr, "driver"));

    let attr: Attribute = parse_quote!(#[not_ragu(driver)]);
    assert!(!attr_is(&attr, "driver"));
}

pub struct GenericDriver {
    pub ident: Ident,
    pub lifetime: Lifetime,
}

impl Default for GenericDriver {
    fn default() -> Self {
        Self {
            ident: format_ident!("D"),
            lifetime: Lifetime::new("'dr", Span::call_site()),
        }
    }
}

impl GenericDriver {
    pub fn extract(generics: &Generics) -> Result<Self> {
        generics
            .params
            .iter()
            .find_map(|p| match p {
                GenericParam::Type(ty) => ty
                    .attrs
                    .iter()
                    .any(|a| attr_is(a, "driver"))
                    .then(|| Self::extract_from_param(ty)),
                _ => None,
            })
            .unwrap_or(Ok(Self::default()))
    }

    fn extract_from_param(param: &TypeParam) -> Result<Self> {
        for bound in &param.bounds {
            if let TypeParamBound::Trait(bound) = bound
                && let Some(seg) = bound.path.segments.last()
            {
                if seg.ident != "Driver" {
                    continue;
                }
                if let PathArguments::AngleBracketed(args) = &seg.arguments {
                    let lifetimes = args
                        .args
                        .iter()
                        .filter_map(|arg| {
                            if let GenericArgument::Lifetime(lt) = arg {
                                Some(lt.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    if lifetimes.len() == 1 {
                        return Ok(GenericDriver {
                            ident: param.ident.clone(),
                            lifetime: lifetimes[0].clone(),
                        });
                    } else {
                        return Err(Error::new(args.span(), "expected a single lifetime bound"));
                    }
                } else {
                    return Err(Error::new(seg.ident.span(), "expected a lifetime bound"));
                }
            }
        }

        Err(Error::new(param.span(), "expected a Driver<'dr> bound"))
    }
}

#[test]
fn test_extract_generic_driver() {
    let generics = parse_quote!(<#[ragu(driver)] D: ragu_core::Driver<'dr>>);
    let driver = GenericDriver::extract(&generics).unwrap();
    assert_eq!(driver.ident.to_string(), "D");
    assert_eq!(driver.lifetime.to_string(), "'dr");

    let generics = parse_quote!(<#[ragu(driver)] D: Driver<'dr>>);
    let driver = GenericDriver::extract(&generics).unwrap();
    assert_eq!(driver.ident.to_string(), "D");
    assert_eq!(driver.lifetime.to_string(), "'dr");

    // Shouldn't cause an error in the macro to have a spurious driver type argument
    let generics = parse_quote!(<#[ragu(driver)] D: Driver<'dr, T>>);
    let driver = GenericDriver::extract(&generics).unwrap();
    assert_eq!(driver.ident.to_string(), "D");
    assert_eq!(driver.lifetime.to_string(), "'dr");

    let generics = parse_quote!(<#[ragu(driver)] D: Driver<'dr, 'another_dr>>);
    assert!(GenericDriver::extract(&generics).is_err());

    let generics = parse_quote!(<#[ragu(driver)] D: Driver>);
    assert!(GenericDriver::extract(&generics).is_err());

    let generics = parse_quote!(<#[ragu(driver)] D: 'a>);
    assert!(GenericDriver::extract(&generics).is_err());

    let generics = parse_quote!(<D: Driver<'dr>>);
    let driver = GenericDriver::extract(&generics).unwrap();
    assert_eq!(driver.ident.to_string(), "D");
    assert_eq!(driver.lifetime.to_string(), "'dr");
}

pub fn macro_body<F>(f: F) -> proc_macro::TokenStream
where
    F: FnOnce() -> Result<TokenStream>,
{
    f().unwrap_or_else(|e| e.into_compile_error()).into()
}
