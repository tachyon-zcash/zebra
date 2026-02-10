//! Stamps and epochs.
//!
//! A stamp bundles everything needed to verify a transaction's
//! validity within the polynomial accumulator system:
//!
//! - **Tachygrams**: Nullifiers and note commitments as polynomial roots
//! - **Proof**: The Ragu proof covering all operations
//! - **Epoch**: The accumulator state anchor

use crate::primitives::{Fp, Tachygram};
use crate::proof::Proof;
use ff::PrimeField;

/// A stamp containing N tachygrams, an N-input proof, and an epoch.
///
/// The const parameter `N` defines both the number of tachygrams and
/// the number of proof inputs.
///
/// Present in [`Autonome`](crate::Autonome) and [`Aggregate`](crate::Aggregate)
/// bundles. Stripped from adjuncts during aggregation.
#[derive(Clone)]
pub struct Stamp<const N: usize> {
    /// N tachygrams (nullifiers and note commitments) for the accumulator.
    tachygrams: [Tachygram; N],

    /// The Ragu proof covering N inputs.
    proof: Proof<N>,

    /// The epoch (recent accumulator state).
    anchor: Epoch,
}

impl<const N: usize> Stamp<N> {
    /// Creates a new stamp.
    pub fn new(tachygrams: [Tachygram; N], proof: Proof<N>, anchor: Epoch) -> Self {
        Self {
            tachygrams,
            proof,
            anchor,
        }
    }

    /// Returns the tachygrams.
    pub fn tachygrams(&self) -> &[Tachygram; N] {
        &self.tachygrams
    }

    /// Returns the proof.
    pub fn proof(&self) -> &Proof<N> {
        &self.proof
    }

    /// Returns the anchor epoch.
    pub fn anchor(&self) -> &Epoch {
        &self.anchor
    }
}

/// The epoch range anchoring an action.
///
/// The anchor identifies a state range for:
/// - Nullifier flavor $\tau$
/// - Proof aggregation by intersection with other anchors
/// - Membership proofs for note commitments (inclusion)
/// - Non-membership proofs for nullifiers (non-inclusion)
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Epoch(pub Fp);
