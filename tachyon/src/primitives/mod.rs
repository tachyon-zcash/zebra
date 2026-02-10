//! Low-level cryptographic primitives for Tachyon.
//!
//! This module provides the fundamental cryptographic building blocks used
//! throughout the Tachyon protocol, built on top of the Ragu proof system
//! and Pasta curves.
//!
//! ## Field Elements
//!
//! Tachyon uses the Pallas curve's base field $\mathbb{F}_p$ as its primary computation
//! field, consistent with the Orchard protocol. The scalar field $\mathbb{F}_q$ is used
//! for scalar operations on the Vesta curve.
//!
//! ## Poseidon Hash
//!
//! The Poseidon algebraic hash function is used for:
//! - Nullifier derivation via the GGM Tree PRF
//! - Note commitments
//! - Accumulator updates

use bitvec::{array::BitArray, order::Lsb0};
use pasta_curves::pallas;

/// The base field of the Pallas curve.
pub type Fp = pallas::Base;

/// The scalar field of the Pallas curve.
pub type Fq = pallas::Scalar;

/// A Pallas point.
pub type PallasPoint = pallas::Affine;

/// The non-negative value of an individual Tachyon note.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct NoteValue(u64);

impl NoteValue {
    pub(crate) fn zero() -> Self {
        // Default for u64 is zero.
        Default::default()
    }

    /// Returns the raw underlying value.
    pub fn inner(&self) -> u64 {
        self.0
    }

    /// Creates a note value from its raw numeric value.
    ///
    /// This only enforces that the value is an unsigned 64-bit integer. Callers should
    /// enforce any additional constraints on the value's valid range themselves.
    pub fn from_raw(value: u64) -> Self {
        NoteValue(value)
    }

    pub(crate) fn from_bytes(bytes: [u8; 8]) -> Self {
        NoteValue(u64::from_le_bytes(bytes))
    }

    pub(crate) fn to_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    pub(crate) fn to_le_bits(self) -> BitArray<[u8; 8], Lsb0> {
        BitArray::<_, Lsb0>::new(self.0.to_le_bytes())
    }
}

/// A tachygram is a 32-byte blob representing either a note commitment
/// or a nullifier in the Tachyon polynomial accumulator.
///
/// The accumulator does not distinguish between commitments and nullifiers.
/// This unified approach simplifies the proof system and enables efficient
/// batch operations.
///
/// Each tachyaction produces exactly one tachygram, regardless of whether
/// it represents a spend (nullifier) or output (commitment) operation.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Tachygram(pub(crate) Fp);

impl From<Tachygram> for Fp {
    fn from(tg: Tachygram) -> Self {
        tg.0
    }
}

/// The epoch range anchoring a stamp.
///
/// The anchor identifies a state range for:
/// - Nullifier flavor $\tau$
/// - Proof aggregation by intersection with other anchors
/// - Membership proofs for note commitments (inclusion)
/// - Non-membership proofs for nullifiers (non-inclusion)
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Epoch(pub Fp);
