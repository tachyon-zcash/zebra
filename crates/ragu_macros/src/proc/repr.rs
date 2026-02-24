use num_bigint::BigUint;
use num_traits::{ToPrimitive, identities::Zero};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, LitInt};

pub fn evaluate(lit: LitInt) -> syn::Result<TokenStream> {
    let n: BigUint = lit.base10_parse()?;
    let mut digits = vec![];
    let mut n = n;
    let mask = num_bigint::BigUint::from(u64::MAX);
    while !n.is_zero() {
        let d = (&n) & &mask;
        digits.push(
            d.to_u64()
                .expect("integer after masking should always be a u64"),
        );
        n >>= 64;
    }
    if digits.len() > 4 {
        return Err(Error::new(lit.span(), "integer larger than 2^256 - 1"));
    } else {
        while digits.len() < 4 {
            digits.push(0);
        }
    }
    Ok(quote!([ #( #digits ),* ]))
}

#[test]
fn test_evaluate() {
    use syn::parse_quote;

    assert!(
        evaluate(parse_quote!(
            0x010000000000000000000000000000000000000000000000000000000000000000
        ))
        .is_err()
    );

    assert_eq!(
        evaluate(parse_quote!(
            0x2c56d224724e82fc9983be57033ecc0b4318967a9394691f790efdd9cee6e373
        ))
        .unwrap()
        .to_string(),
        quote!([
            8723188640184198003u64,
            4834779653188380959u64,
            11061894390677949451u64,
            3194972039644349180u64
        ])
        .to_string()
    );

    assert_eq!(
        evaluate(parse_quote!(0)).unwrap().to_string(),
        quote!([0u64, 0u64, 0u64, 0u64]).to_string()
    );

    let max = u64::MAX;

    assert_eq!(
        evaluate(parse_quote!(
            0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
        ))
        .unwrap()
        .to_string(),
        quote!([
            #max,
            #max,
            #max,
            #max
        ])
        .to_string()
    );
}
