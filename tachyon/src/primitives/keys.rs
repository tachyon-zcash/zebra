use crate::primitives::{Fp, Fq, SigningKey, SpendAuth, Tachygram};
use ff::{Field, PrimeField};

/// A Tachyon spending key.
///
/// The root key from which all other keys are derived. This key must
/// be kept secret as it provides full spending authority.
#[derive(Clone)]
pub struct SpendingKey(Fp);

impl SpendingKey {
    /// Derives the spend authorizing key `ask` from this spending key.
    ///
    /// The returned [`SigningKey<SpendAuth>`] can be randomized per-action
    /// to produce a signing key whose corresponding verification key is
    /// the `rk` in each [`Action`](crate::Action).
    pub fn spend_authorizing_key(&self) -> SigningKey<SpendAuth> {
        // TODO: Implement proper PRF-based derivation
        let bytes = self.0.to_repr();
        bytes.try_into().expect("valid signing key")
    }

    /// Derives the nullifier key from this spending key.
    pub fn nullifier_key(&self) -> NullifierKey {
        // TODO: Implement proper key derivation using Poseidon
        NullifierKey(self.0)
    }

    /// Derives the payment key from this spending key.
    pub fn payment_key(&self) -> PaymentKey {
        // TODO: Implement proper key derivation
        PaymentKey(self.0)
    }
}

/// A Tachyon nullifier key.
///
/// Used for nullifier derivation via a GGM tree PRF instantiated from
/// Poseidon.
#[derive(Clone)]
pub struct NullifierKey(Fp);

impl NullifierKey {
    /// Derives a nullifier tachygram from this key, a trapdoor, and an epoch.
    pub fn derive(&self, _trapdoor: &Fq, _epoch: crate::stamp::Epoch) -> Tachygram {
        // TODO: Implement actual Poseidon-based GGM tree PRF
        Tachygram(Fp::ZERO)
    }
}

/// A Tachyon payment key.
///
/// Used in note construction and out-of-band payment protocols.
#[derive(Clone)]
pub struct PaymentKey(Fp);
