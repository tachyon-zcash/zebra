//! Tachygrams - unified commitment/nullifier representation.
//!
//! Unlike Orchard which maintains separate trees for note commitments and nullifiers,
//! Tachyon uses a unified polynomial accumulator that tracks both via tachygrams.
//! This enables more efficient proofs in the recursive (PCD) context.

use crate::note::{NoteCommitment, Nullifier};
use crate::primitives::Fp;

/// A tachygram is a generic data string representing either a note commitment
/// or a nullifier in the Tachyon polynomial accumulator.
///
/// The accumulator does not distinguish between commitments and nullifiers;
/// both are treated as field elements to be accumulated. This unified approach
/// simplifies the proof system and enables efficient batch operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tachygram(pub Fp);


impl From<NoteCommitment> for Tachygram {
    fn from(commitment: NoteCommitment) -> Self {
        Self(commitment.0)
    }
}

impl From<Nullifier> for Tachygram {
    fn from(nullifier: Nullifier) -> Self {
        Self(nullifier.0)
    }
}
