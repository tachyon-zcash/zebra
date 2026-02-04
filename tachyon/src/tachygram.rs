//! Tachygrams - unified commitment/nullifier representation.
//!
//! **Tachygrams** are 32-byte blobs that represent either nullifiers or note
//! commitments. The consensus protocol does not distinguish between them—they
//! are treated identically and inserted in a single polynomial accumulator.
//!
//! This unification simplifies the protocol and enables efficient
//! membership/non-membership proofs. Unlike Orchard which maintains separate
//! trees for note commitments and nullifiers, Tachyon uses a unified
//! polynomial accumulator that tracks both via tachygrams.
//!
//! ## Operations
//!
//! - **Spend operation**: Consumes a note by publishing a nullifier (as tachygram)
//! - **Output operation**: Creates a note by publishing a commitment (as tachygram)
//!
//! It is intentionally indistinguishable whether a tachygram represents a
//! nullifier or a commitment—this provides additional privacy.

use crate::note::{NoteCommitment, Nullifier};
use crate::primitives::Fp;

/// A tachygram is a 32-byte blob representing either a note commitment
/// or a nullifier in the Tachyon polynomial accumulator.
///
/// The accumulator does not distinguish between commitments and nullifiers;
/// both are treated as field elements to be accumulated. This unified approach
/// simplifies the proof system and enables efficient batch operations.
///
/// Each tachyaction produces exactly one tachygram, regardless of whether
/// it represents a spend (nullifier) or output (commitment) operation.
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
