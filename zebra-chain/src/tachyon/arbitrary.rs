//! Proptest arbitrary implementations for Tachyon types.
//!
//! These implement `Arbitrary` for tachyon crate types (orphan rule is
//! satisfied because proptest is a dev-dependency) and for zebra's
//! `ShieldedData`.

use proptest::{arbitrary::any, prelude::*};

use halo2::pasta::pallas;

use crate::amount::Amount;

use super::ShieldedData;

impl Arbitrary for ShieldedData {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (
            proptest::collection::vec(arb_action(), 0..5),
            any::<i64>(),
            proptest::option::of(arb_stamp()),
        )
            .prop_map(|(actions, vb, stamp)| {
                let value_balance = Amount::try_from(vb % 1000).unwrap_or_else(|_| Amount::zero());
                let binding_sig = if !actions.is_empty() {
                    Some(zcash_tachyon::BindingSignature::from([0u8; 64]))
                } else {
                    None
                };
                ShieldedData {
                    actions,
                    value_balance,
                    binding_sig,
                    stamp,
                }
            })
            .boxed()
    }
}

fn arb_action() -> impl Strategy<Value = zcash_tachyon::Action> {
    (any::<bool>(), any::<[u8; 32]>(), any::<[u8; 64]>()).prop_map(
        |(use_identity, rk_bytes, sig_bytes)| {
            let cv = if use_identity {
                super::ValueCommitment::balance(0)
            } else {
                super::ValueCommitment::balance(1)
            };
            // Fallback to a known-good key if random bytes are invalid
            let rk = zcash_tachyon::RandomizedVerificationKey::try_from(rk_bytes).unwrap_or_else(|_| {
                zcash_tachyon::RandomizedVerificationKey::try_from([1u8; 32]).unwrap()
            });
            let sig = zcash_tachyon::SpendAuthSignature::from(sig_bytes);
            zcash_tachyon::Action { cv, rk, sig }
        },
    )
}

fn arb_stamp() -> impl Strategy<Value = zcash_tachyon::Stamp> {
    (
        proptest::collection::vec(arb_tachygram(), 0..20),
        arb_anchor(),
    )
        .prop_map(|(tachygrams, anchor)| zcash_tachyon::Stamp {
            tachygrams,
            anchor,
            proof: zcash_tachyon::Proof::default(),
        })
}

fn arb_tachygram() -> impl Strategy<Value = zcash_tachyon::Tachygram> {
    any::<u64>().prop_map(|val| zcash_tachyon::Tachygram::from(pallas::Base::from(val)))
}

fn arb_anchor() -> impl Strategy<Value = zcash_tachyon::Anchor> {
    any::<u64>().prop_map(|val| zcash_tachyon::Anchor::from(pallas::Base::from(val)))
}
