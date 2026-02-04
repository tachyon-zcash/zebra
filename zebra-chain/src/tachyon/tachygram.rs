//! Tachygram: A unified field element for nullifiers and note commitments.
//!
//! Tachygrams form the basis of the Tachyon polynomial accumulator, enabling a single
//! structure for both nullifiers and note commitments.

use std::{
    fmt,
    hash::{Hash, Hasher},
};

use group::ff::PrimeField;
use halo2::pasta::pallas;

use crate::serialization::serde_helpers;

use super::commitment::NoteCommitment;

/// A unified field element that can be either a nullifier or note commitment.
///
/// Tachygrams are roots of the polynomial in the Tachyon accumulator. Both nullifiers
/// and note commitments are field elements, enabling efficient set (non-)membership proofs.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tachygram(#[serde(with = "serde_helpers::Base")] pub pallas::Base);

impl Hash for Tachygram {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_repr().hash(state);
    }
}

impl Tachygram {
    /// The size of a serialized Tachygram in bytes.
    pub const SIZE: usize = 32;
}

impl From<&tachyon::Nullifier> for Tachygram {
    fn from(nf: &tachyon::Nullifier) -> Self {
        Self(nf.0)
    }
}

impl From<&NoteCommitment> for Tachygram {
    fn from(cm: &NoteCommitment) -> Self {
        Self(cm.extract_x())
    }
}

impl fmt::Debug for Tachygram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Tachygram")
            .field(&hex::encode(self.0.to_repr()))
            .finish()
    }
}

impl From<tachyon::Tachygram> for Tachygram {
    fn from(tg: tachyon::Tachygram) -> Self {
        Self(tg.0)
    }
}
