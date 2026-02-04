//! Tachyon polynomial accumulator.
//!
//! **IMPORTANT**: Unlike Sapling/Orchard which use Merkle trees, Tachyon uses a
//! **polynomial accumulator** where tachygrams are roots of a polynomial.
//! There is NO tree Root type.
//!
//! From the spec:
//! > "The accumulator will be a commitment to a polynomial with roots at the
//! > committed values, hashed with the previous accumulator value."
//!
//! This enables efficient set membership and non-membership proofs using
//! polynomial evaluation, which integrates with the Ragu PCD system.
//!
//! Tachygrams themselves serve as membership witnesses - the Ragu proof system
//! verifies that a tachygram is a root of the polynomial.
//!
//! ## Types
//!
//! - [`Anchor`] - serializable accumulator anchor for blockchain storage
//! - [`tachyon::Epoch`] - protocol-level epoch (anchor) type from the tachyon crate
//!
//! Use conversion methods to move between these representations.

use std::{
    fmt,
    hash::{Hash, Hasher},
    io,
};

use group::ff::PrimeField;
use halo2::pasta::pallas;

use crate::serialization::{
    serde_helpers, ReadZcashExt, SerializationError, ZcashDeserialize, ZcashSerialize,
};

/// Tachyon accumulator anchor (placeholder).
///
/// This represents the current state of the polynomial accumulator.
/// Unlike a Merkle tree root, this is a polynomial commitment where
/// tachygrams are the roots of the committed polynomial.
///
/// **TODO**: This is a placeholder type. The actual anchor representation
/// will be defined by the Ragu accumulator integration.
#[derive(Clone, Copy, Eq, Serialize, Deserialize)]
pub struct Anchor(#[serde(with = "serde_helpers::Base")] pub pallas::Base);

impl Anchor {
    /// The size of a serialized Anchor in bytes.
    pub const SIZE: usize = 32;
}

impl From<pallas::Base> for Anchor {
    fn from(base: pallas::Base) -> Self {
        Self(base)
    }
}

impl From<Anchor> for pallas::Base {
    fn from(anchor: Anchor) -> Self {
        anchor.0
    }
}

impl fmt::Debug for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("tachyon::accumulator::Anchor")
            .field(&hex::encode(self.0.to_repr()))
            .finish()
    }
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0.to_repr()))
    }
}

impl PartialEq for Anchor {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for Anchor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_repr().hash(state);
    }
}

impl From<Anchor> for [u8; 32] {
    fn from(anchor: Anchor) -> Self {
        anchor.0.to_repr()
    }
}

impl TryFrom<[u8; 32]> for Anchor {
    type Error = SerializationError;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        let base = pallas::Base::from_repr(bytes);
        if base.is_some().into() {
            Ok(Self(base.unwrap()))
        } else {
            Err(SerializationError::Parse(
                "Invalid pallas::Base value for Tachyon accumulator anchor",
            ))
        }
    }
}

impl ZcashSerialize for Anchor {
    fn zcash_serialize<W: io::Write>(&self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&self.0.to_repr())
    }
}

impl ZcashDeserialize for Anchor {
    fn zcash_deserialize<R: io::Read>(mut reader: R) -> Result<Self, SerializationError> {
        Self::try_from(reader.read_32_bytes()?)
    }
}

impl From<tachyon::Epoch> for Anchor {
    fn from(epoch: tachyon::Epoch) -> Self {
        // Convert the tachyon Epoch bytes to pallas::Base
        let bytes = epoch.to_bytes();
        Self(pallas::Base::from_repr(bytes).expect("valid field element from tachyon::Epoch"))
    }
}

impl From<Anchor> for tachyon::Epoch {
    fn from(anchor: Anchor) -> Self {
        tachyon::Epoch::from_bytes(&anchor.0.to_repr()).expect("valid field element from Anchor")
    }
}

// Note: The actual polynomial accumulator implementation will be provided by
// the Ragu library. The accumulator state is:
//
//   new_anchor = hash(polynomial_commitment(tachygrams), previous_anchor)
//
// Where polynomial_commitment creates a polynomial P(x) such that
// P(tachygram) = 0 for all committed tachygrams.
//
// Set membership proofs show that a tachygram is a root of the polynomial.
// Set non-membership proofs show that a tachygram is NOT a root.
