//! Tachygram: A unified field element for nullifiers and note commitments.
//!
//! **Tachygrams** are 32-byte blobs that represent either nullifiers or note
//! commitments. The consensus protocol does not distinguish between them—they
//! are treated identically and inserted in a single polynomial accumulator.
//!
//! This unification simplifies the protocol and enables efficient
//! membership/non-membership proofs.

use std::{
    fmt,
    hash::{Hash, Hasher},
};

use group::ff::PrimeField;
use halo2::pasta::pallas;

use crate::serialization::serde_helpers;

use super::commitment::NoteCommitment;

/// A 32-byte blob representing either a nullifier or note commitment.
///
/// Tachygrams are roots of the polynomial in the Tachyon accumulator.
/// The accumulator does not distinguish between commitments and nullifiers;
/// both are treated as field elements. This unified approach simplifies
/// the proof system and enables efficient batch operations.
///
/// Each tachyaction produces exactly one tachygram, and it is intentionally
/// indistinguishable whether that tachygram represents a nullifier or
/// commitment—this provides additional privacy.
#[derive(Clone, Copy, Serialize, Deserialize)]
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
