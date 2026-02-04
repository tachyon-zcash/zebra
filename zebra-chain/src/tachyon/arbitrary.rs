//! Proptest arbitrary implementations for Tachyon types.

use group::prime::PrimeCurveAffine;
use halo2::pasta::pallas;
use proptest::{arbitrary::any, prelude::*};
use reddsa::Signature;

use super::{
    accumulator::Epoch,
    action::Tachyaction,
    commitment::ValueCommitment,
    proof::Proof,
    shielded_data::Tachystamp,
    tachygram::Tachygram,
};

impl Arbitrary for Tachygram {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        any::<u64>()
            .prop_map(|val| Tachygram(pallas::Base::from(val)))
            .boxed()
    }
}

impl Arbitrary for ValueCommitment {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        // Generate a random point on the curve by using identity or generator
        // For testing purposes, we use simple deterministic values
        any::<bool>()
            .prop_map(|use_identity| {
                if use_identity {
                    ValueCommitment(pallas::Affine::identity())
                } else {
                    ValueCommitment(pallas::Affine::generator())
                }
            })
            .boxed()
    }
}

impl Arbitrary for Tachyaction {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (any::<ValueCommitment>(), any::<[u8; 32]>(), any::<[u8; 64]>())
            .prop_map(|(cv, rk_bytes, sig_bytes)| Tachyaction {
                cv,
                rk: rk_bytes.into(),
                spend_auth_sig: Signature::from(sig_bytes),
            })
            .boxed()
    }
}

impl Arbitrary for Proof {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        proptest::collection::vec(any::<u8>(), 0..256)
            .prop_map(|bytes| Proof::new(bytes).unwrap())
            .boxed()
    }
}

impl Arbitrary for Tachystamp {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (
            proptest::collection::vec(any::<Tachygram>(), 0..20),
            any::<Proof>(),
            any::<Epoch>(),
        )
            .prop_map(|(tachygrams, proof, anchor)| Tachystamp::new(tachygrams, proof, anchor))
            .boxed()
    }
}

impl Arbitrary for Epoch {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        any::<u64>()
            .prop_map(|val| Epoch::from(pallas::Base::from(val)))
            .boxed()
    }
}
