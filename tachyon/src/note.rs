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

/// The epoch (accumulator anchor) for Tachyon transactions.
///
/// The epoch is the polynomial commitment representing the accumulator state:
/// - Identifies the state of the polynomial accumulator at a point in time
/// - Enables membership proofs for tachygrams
/// - Used as the "flavor" in nullifier derivation
///
/// Epochs are valid within a range and can be accumulated to a single epoch
/// during proof aggregation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Epoch(pub Fp);

impl Epoch {
    /// Returns the byte representation of this epoch.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_repr()
    }

    /// Creates an epoch from bytes.
    ///
    /// Returns `None` if the bytes do not represent a valid field element.
    pub fn from_bytes(bytes: &[u8; 32]) -> Option<Self> {
        Fp::from_repr(*bytes).map(Self).into()
    }
}

impl std::fmt::Display for Epoch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = self.to_bytes();
        for b in &bytes[..8] {
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

impl Nullifier {
    /// Returns the byte representation of this nullifier.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_repr()
    }

    /// Creates a nullifier from bytes.
    ///
    /// Returns `None` if the bytes do not represent a valid field element.
    pub fn from_bytes(bytes: &[u8; 32]) -> Option<Self> {
        Fp::from_repr(*bytes).map(Self).into()
    }
}

/// A note commitment.
///
/// This is a hiding commitment to a note, stored in the polynomial accumulator
/// as a tachygram.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoteCommitment(pub Fp);

impl NoteCommitment {
    /// Returns the byte representation.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_repr()
    }
}
