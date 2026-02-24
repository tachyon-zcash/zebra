/// Creates a raw [`Fp`](pasta_curves::Fp) element from a hex string literal
#[macro_export]
macro_rules! fp {
    ( $x:expr ) => {
        $crate::Fp::from_raw(ragu_arithmetic::repr256!($x))
    };
}

/// Creates a raw [`Fq`](pasta_curves::Fq) element from a hex string literal
#[macro_export]
macro_rules! fq {
    ( $x:expr ) => {
        $crate::Fq::from_raw(ragu_arithmetic::repr256!($x))
    };
}
