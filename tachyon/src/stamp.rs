//! Stamps and epochs.
//!
//! A stamp bundles everything needed to verify a transaction's
//! validity within the polynomial accumulator system:
//!
//! - **Tachygrams**: Nullifiers and note commitments as polynomial roots
//! - **Proof**: The Ragu proof covering all operations
//! - **Epoch**: The accumulator state anchor

use crate::primitives::{Epoch, Tachygram};
use crate::proof::Proof;

/// A stamp containing tachygrams, a proof, and an epoch.
///
/// Present in [`StampedBundle`](crate::StampedBundle) bundles.
/// Stripped from adjuncts during aggregation.
#[derive(Clone)]
pub struct Stamp {
    /// Tachygrams (nullifiers and note commitments) for the accumulator.
    pub(crate) tachygrams: Vec<Tachygram>,

    /// The Ragu proof covering all inputs.
    pub(crate) proof: Proof,

    /// The epoch (recent accumulator state).
    pub(crate) anchor: Epoch,
}
