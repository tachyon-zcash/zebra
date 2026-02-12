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

use pasta_curves::pallas;

/// The base field of the Pallas curve.
pub type Fp = pallas::Base;

/// The scalar field of the Pallas curve.
pub type Fq = pallas::Scalar;

/// A Pallas point.
pub type PallasPoint = pallas::Affine;

/// A tachygram is a 32-byte blob representing either a note commitment
/// or a nullifier in the Tachyon polynomial accumulator.
///
/// The accumulator does not distinguish between commitments and nullifiers.
/// This unified approach simplifies the proof system and enables efficient
/// batch operations.
///
/// Each tachyaction produces exactly one tachygram, regardless of whether
/// it represents a spend (nullifier) or output (commitment) operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tachygram(pub Fp);

impl From<Tachygram> for Fp {
    fn from(tg: Tachygram) -> Self {
        tg.0
    }
}

impl From<Tachygram> for [u8; 32] {
    fn from(tg: Tachygram) -> Self {
        use group::ff::PrimeField;
        tg.0.to_repr()
    }
}

impl TryFrom<[u8; 32]> for Tachygram {
    type Error = &'static str;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        use group::ff::PrimeField;
        let elem = Fp::from_repr(bytes);
        if elem.is_some().into() {
            Ok(Self(elem.unwrap()))
        } else {
            Err("Invalid pallas::Base for Tachygram")
        }
    }
}

/// The epoch range anchoring a stamp.
///
/// The anchor identifies a state range for:
/// - Nullifier flavor $\tau$
/// - Proof aggregation by intersection with other anchors
/// - Membership proofs for note commitments (inclusion)
/// - Non-membership proofs for nullifiers (non-inclusion)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Epoch(pub Fp);
