//! Tachyon nullifiers with epoch "flavoring".
//!
//! **NOTE**: This module may be deprecated. In Tachyon, nullifiers become
//! tachygrams in the polynomial accumulator. The separate `Nullifier` and
//! `FlavoredNullifier` types may be unified into `Tachygram`.
//!
//! This module provides [`FlavoredNullifier`], a serializable wrapper that bundles
//! a [`tachyon::Nullifier`] with its associated [`Epoch`](tachyon::Epoch) for
//! blockchain storage.

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

/// A Tachyon nullifier bundled with its epoch flavor for blockchain serialization.
///
/// **NOTE**: This type may be deprecated in favor of `Tachygram`. In Tachyon,
/// nullifiers and note commitments are unified as tachygrams in the polynomial
/// accumulator.
///
/// This type pairs a [`tachyon::Nullifier`] with an [`tachyon::Epoch`] to enable:
/// - Blockchain serialization (both fields need to be persisted)
/// - Epoch tracking for nullifier derivation
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct FlavoredNullifier {
    /// The nullifier value as a field element.
    #[serde(with = "serde_helpers::Base")]
    nf: pallas::Base,

    /// The epoch (accumulator anchor) as a field element.
    #[serde(with = "serde_helpers::Base")]
    epoch: pallas::Base,
}

impl FlavoredNullifier {
    /// The size of a serialized FlavoredNullifier in bytes (32 for nf + 32 for epoch).
    pub const SIZE: usize = 64;

    /// Create a new FlavoredNullifier from its components.
    pub fn new(nullifier: tachyon::Nullifier, epoch: tachyon::Epoch) -> Self {
        Self {
            nf: nullifier.0,
            epoch: epoch.0,
        }
    }

    /// Get the nullifier value.
    pub fn nullifier(&self) -> tachyon::Nullifier {
        tachyon::Nullifier(self.nf)
    }

    /// Get the epoch (accumulator anchor).
    pub fn epoch(&self) -> tachyon::Epoch {
        tachyon::Epoch(self.epoch)
    }

    /// Get the nullifier value as bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.nf.to_repr()
    }

    /// Try to create a FlavoredNullifier from bytes and an epoch.
    pub fn try_from_bytes(
        bytes: [u8; 32],
        epoch: tachyon::Epoch,
    ) -> Result<Self, SerializationError> {
        let nf = pallas::Base::from_repr(bytes);
        if nf.is_some().into() {
            Ok(Self {
                nf: nf.unwrap(),
                epoch: epoch.0,
            })
        } else {
            Err(SerializationError::Parse(
                "Invalid field element for Tachyon Nullifier",
            ))
        }
    }
}

impl fmt::Debug for FlavoredNullifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlavoredNullifier")
            .field("nf", &hex::encode(self.nf.to_repr()))
            .field("epoch", &hex::encode(self.epoch.to_repr()))
            .finish()
    }
}

impl PartialEq for FlavoredNullifier {
    fn eq(&self, other: &Self) -> bool {
        self.nf == other.nf && self.epoch == other.epoch
    }
}

impl Eq for FlavoredNullifier {}

impl Hash for FlavoredNullifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nf.to_repr().hash(state);
        self.epoch.to_repr().hash(state);
    }
}

impl ZcashSerialize for FlavoredNullifier {
    fn zcash_serialize<W: io::Write>(&self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&self.nf.to_repr())?;
        writer.write_all(&self.epoch.to_repr())?;
        Ok(())
    }
}

impl ZcashDeserialize for FlavoredNullifier {
    fn zcash_deserialize<R: io::Read>(mut reader: R) -> Result<Self, SerializationError> {
        let nf_bytes = reader.read_32_bytes()?;
        let nf = pallas::Base::from_repr(nf_bytes);
        let nf = if nf.is_some().into() {
            nf.unwrap()
        } else {
            return Err(SerializationError::Parse(
                "Invalid field element for Tachyon Nullifier",
            ));
        };

        let epoch_bytes = reader.read_32_bytes()?;
        let epoch = pallas::Base::from_repr(epoch_bytes);
        let epoch = if epoch.is_some().into() {
            epoch.unwrap()
        } else {
            return Err(SerializationError::Parse(
                "Invalid field element for Tachyon Epoch",
            ));
        };

        Ok(Self { nf, epoch })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tachyon::primitives::Fp;

    #[test]
    fn flavored_nullifier_serialization_roundtrip() {
        let _init_guard = zebra_test::init();

        let nf = FlavoredNullifier::new(
            tachyon::Nullifier(Fp::from(12345u64)),
            tachyon::Epoch(Fp::from(42u64)),
        );

        let mut bytes = Vec::new();
        nf.zcash_serialize(&mut bytes).unwrap();

        assert_eq!(bytes.len(), FlavoredNullifier::SIZE);

        let nf2 = FlavoredNullifier::zcash_deserialize(&bytes[..]).unwrap();
        assert_eq!(nf, nf2);
    }
}
