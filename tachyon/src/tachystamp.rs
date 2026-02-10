//! Tachystamp: proof, tachygrams, and epoch.
//!
//! The tachystamp bundles everything needed to verify a transaction's
//! validity within the polynomial accumulator system:
//!
//! - **Tachygrams**: Nullifiers and note commitments as polynomial roots
//! - **Proof**: The Ragu proof covering all operations
//! - **Epoch**: The accumulator state anchor

use crate::Tachygram;
use crate::note::Epoch;
use crate::proof::Proof;

/// Tachystamp containing the proof, tachygrams, and epoch.
///
/// This type bundles:
/// - All tachygrams (nullifiers and note commitments) for the transaction
/// - The Ragu proof proving validity of all operations
/// - The epoch (accumulator anchor)
///
/// Present in [`Autonome`](crate::Autonome) bundles (self-contained) and
/// [`Aggregate`](crate::Aggregate) bundles (covering multiple
/// [`Adjunct`](crate::Adjunct) bundles).
///
/// Epochs from multiple tachystamps can be accumulated into a single epoch
/// during proof aggregation.
#[derive(Clone, Debug)]
pub struct Tachystamp {
    /// All tachygrams from this transaction.
    ///
    /// These are the nullifiers and note commitments that get recorded
    /// in the polynomial accumulator.
    tachygrams: Vec<Tachygram>,

    /// The Ragu proof covering all operations.
    proof: Proof,

    /// The epoch (recent accumulator state).
    ///
    /// All spends in this transaction reference notes committed at or
    /// before this accumulator state. Epochs are valid within a range.
    anchor: Epoch,
}

impl Tachystamp {
    /// Creates a new tachystamp.
    pub fn new(tachygrams: Vec<Tachygram>, proof: Proof, anchor: Epoch) -> Self {
        Self {
            tachygrams,
            proof,
            anchor,
        }
    }

    /// Returns the tachygrams in this tachystamp.
    pub fn tachygrams(&self) -> &[Tachygram] {
        &self.tachygrams
    }

    /// Returns the proof.
    pub fn proof(&self) -> &Proof {
        &self.proof
    }

    /// Returns the anchor epoch.
    pub fn anchor(&self) -> &Epoch {
        &self.anchor
    }

    /// Returns the number of tachygrams in this tachystamp.
    pub fn tachygram_count(&self) -> usize {
        self.tachygrams.len()
    }
}
