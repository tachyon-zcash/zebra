//! Tachyon notes and nullifiers.
//!
//! Notes represent discrete amounts of value within the Tachyon shielded pool.
//! When spent, notes produce nullifiers that are tracked via tachygrams in the
//! polynomial accumulator.

use ff::PrimeField;

use crate::primitives::Fp;

/// A Tachyon note.
///
/// Notes represent discrete amounts of value within the Tachyon shielded pool.
/// Each note contains:
/// - A value amount
/// - A recipient address
/// - Randomness for hiding the note commitment
///
/// This is a stub type for progressive implementation.
#[derive(Clone, Debug)]
pub struct Note {
    /// The value of this note in zatoshis.
    pub value: u64,
    /// Random seed for note randomness derivation.
    pub rseed: Fp,
}

/// The nullifier trapdoor $\Psi$ for a Tachyon note.
///
/// This is user-controlled randomness that, combined with the nullifier key
/// and epoch flavor, produces a unique nullifier for the note.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NullifierTrapdoor(pub Fp);

/// The epoch range anchoring a tachyaction.
///
/// The anchor identifies a state range for:
/// - Nullifier flavor $\tau$
/// - Proof aggregation by intersection with other anchors
/// - Membership proofs for note commitments (inclusion)
/// - Non-membership proofs for nullifiers (non-inclusion)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Epoch(pub Fp);

impl std::fmt::Display for Epoch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for b in &self.0.to_repr()[..8] {
            write!(f, "{:02x}", b)?;
        }
        write!(f, "...")
    }
}

/// A nullifier for a spent Tachyon note.
///
/// Tachyon uses a simplified nullifier design compared to Orchard:
///
/// $$\mathsf{nf} = F_{\mathsf{nk}}(\Psi \| e)$$
///
/// where:
/// - $\mathsf{nk}$ is the nullifier key
/// - $\Psi$ is the nullifier trapdoor (user-controlled randomness)
/// - $e$ is the epoch (flavor)
///
/// This design enables constrained PRFs for oblivious syncing delegation:
/// a delegated key $\mathsf{nk}_t$ can only compute nullifiers for epochs $e \leq t$.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Nullifier(pub Fp);

/// A note commitment.
///
/// This is a hiding commitment to a note, stored in the polynomial accumulator
/// as a tachygram.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoteCommitment(pub Fp);
