//! Tachyon polynomial accumulator.
//!
//! **IMPORTANT**: Unlike Sapling/Orchard which use Merkle trees, Tachyon uses a
//! **polynomial accumulator** where tachygrams are roots of a polynomial.
//! There is NO tree Root type.
//!
//! From the spec:
//! > "The accumulator will be a commitment to a polynomial with roots at the
//! > committed values, hashed with the previous accumulator value."
//!
//! This enables efficient set membership and non-membership proofs using
//! polynomial evaluation, which integrates with the Ragu PCD system.
//!
//! Tachygrams themselves serve as membership witnesses - the Ragu proof system
//! verifies that a tachygram is a root of the polynomial.

use std::hash::{Hash, Hasher};

use group::ff::PrimeField;
use halo2::pasta::pallas;

use crate::serialization::serde_helpers;

/// The epoch (accumulator state) for Tachyon transactions.
///
/// This represents the current state of the polynomial accumulator.
/// Unlike a Merkle tree root, this is a polynomial commitment where
/// tachygrams are the roots of the committed polynomial.
///
/// The epoch is used:
/// - As the "flavor" in nullifier derivation
/// - As the anchor for membership proofs in transactions
/// - To identify accumulator state at a point in time
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Epoch(#[serde(with = "serde_helpers::Base")] pub pallas::Base);

impl Epoch {
    /// The size of a serialized Epoch in bytes.
    pub const SIZE: usize = 32;
}

impl std::fmt::Debug for Epoch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("tachyon::Epoch")
            .field(&hex::encode(self.0.to_repr()))
            .finish()
    }
}

impl std::fmt::Display for Epoch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0.to_repr()))
    }
}

impl Hash for Epoch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_repr().hash(state);
    }
}

impl From<pallas::Base> for Epoch {
    fn from(base: pallas::Base) -> Self {
        Self(base)
    }
}

impl From<tachyon::Epoch> for Epoch {
    fn from(epoch: tachyon::Epoch) -> Self {
        Self(epoch.0)
    }
}
