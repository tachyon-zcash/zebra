use proptest::prelude::*;

use super::ShieldedTransactionAggregate;

impl Arbitrary for ShieldedTransactionAggregate {
    type Parameters = ();

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        Just(ShieldedTransactionAggregate {}).boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}
