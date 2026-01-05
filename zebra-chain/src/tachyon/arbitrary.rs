use halo2::pasta::pallas;
use proptest::prelude::*;

use super::{ShieldedTransactionAggregate, Tachygram};

impl Arbitrary for ShieldedTransactionAggregate {
    type Parameters = ();

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        Just(ShieldedTransactionAggregate {}).boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

impl Arbitrary for Tachygram {
    type Parameters = ();

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        // Use the generator point as a placeholder for arbitrary tachygrams
        Just(Tachygram(pallas::Affine::generator())).boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}
