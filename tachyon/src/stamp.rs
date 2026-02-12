//! Stamps and epochs.
//!
//! A stamp bundles everything needed to verify a transaction's
//! validity within the polynomial accumulator system:
//!
//! - **Tachygrams**: Nullifiers and note commitments as polynomial roots
//! - **Proof**: The Ragu proof covering all operations
//! - **Epoch**: The accumulator state anchor

use crate::action::Action;
use crate::primitives::{Epoch, Tachygram};
use crate::proof::{ActionWitness, Proof};

/// A stamp containing tachygrams, a proof, and an epoch.
///
/// Present in [`StampedBundle`](crate::StampedBundle) bundles.
/// Stripped from adjuncts during aggregation.
#[derive(Clone, Debug)]
pub struct Stamp {
    /// Tachygrams (nullifiers and note commitments) for the accumulator.
    pub tachygrams: Vec<Tachygram>,

    /// The Ragu proof covering all inputs.
    pub proof: Proof,

    /// The epoch (recent accumulator state).
    pub anchor: Epoch,
}

impl Stamp {
    /// Creates a stamp by running the proof black box over action witnesses.
    ///
    /// The proof system produces tachygrams (rerandomized nullifiers and
    /// note commitments) alongside the proof.
    pub fn prove(witnesses: Vec<ActionWitness>, actions: Vec<Action>, anchor: Epoch) -> Self {
        let (proof, tachygrams) = Proof::create(&witnesses, &actions, &anchor);
        Stamp {
            tachygrams,
            proof,
            anchor,
        }
    }

    /// Merges this stamp with another, combining tachygrams and proofs.
    ///
    /// Both stamps must share the same anchor (epoch).
    /// The proof merge is a black box operation (Ragu PCD fuse).
    pub fn merge(self, other: Stamp) -> Self {
        assert_eq!(
            self.anchor, other.anchor,
            "anchors must match for stamp merge"
        );
        let mut tachygrams = self.tachygrams;
        tachygrams.extend(other.tachygrams);
        let proof = Proof::merge(self.proof, other.proof);
        Stamp {
            tachygrams,
            proof,
            anchor: self.anchor,
        }
    }
}
