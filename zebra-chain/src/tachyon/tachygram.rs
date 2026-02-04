//! Tachygram: A unified field element for nullifiers and note commitments.
//!
//! Tachygrams form the basis of the Tachyon polynomial accumulator, enabling a single
//! structure for both nullifiers and note commitments.

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

use super::commitment::NoteCommitment;

/// A unified field element that can be either a nullifier or note commitment.
///
/// Tachygrams are roots of the polynomial in the Tachyon accumulator. Both nullifiers
/// and note commitments are field elements, enabling efficient set (non-)membership proofs.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tachygram(#[serde(with = "serde_helpers::Base")] pub pallas::Base);

impl Hash for Tachygram {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_repr().hash(state);
    }
}

impl Tachygram {
    /// The size of a serialized Tachygram in bytes.
    pub const SIZE: usize = 32;

    /// Create a Tachygram from a Tachyon nullifier.
    pub fn from_nullifier(nf: &tachyon::Nullifier) -> Self {
        Self(nf.0)
    }

    /// Create a Tachygram from a Tachyon note commitment.
    pub fn from_note_commitment(cm: &NoteCommitment) -> Self {
        Self(cm.extract_x())
    }
}

impl fmt::Debug for Tachygram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Tachygram")
            .field(&hex::encode(self.0.to_repr()))
            .finish()
    }
}

impl From<pallas::Base> for Tachygram {
    fn from(value: pallas::Base) -> Self {
        Self(value)
    }
}

impl From<Tachygram> for pallas::Base {
    fn from(tg: Tachygram) -> Self {
        tg.0
    }
}

impl From<tachyon::Tachygram> for Tachygram {
    fn from(tg: tachyon::Tachygram) -> Self {
        Self(tg.0)
    }
}

impl From<Tachygram> for tachyon::Tachygram {
    fn from(tg: Tachygram) -> Self {
        tachyon::Tachygram(tg.0)
    }
}

impl ZcashSerialize for Tachygram {
    fn zcash_serialize<W: io::Write>(&self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&self.0.to_repr())
    }
}

impl ZcashDeserialize for Tachygram {
    fn zcash_deserialize<R: io::Read>(mut reader: R) -> Result<Self, SerializationError> {
        let bytes = reader.read_32_bytes()?;
        let base = pallas::Base::from_repr(bytes);
        if base.is_some().into() {
            Ok(Self(base.unwrap()))
        } else {
            Err(SerializationError::Parse(
                "Invalid field element for Tachygram",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tachygram_roundtrip() {
        let _init_guard = zebra_test::init();

        let tg = Tachygram(pallas::Base::from(42u64));

        let mut bytes = Vec::new();
        tg.zcash_serialize(&mut bytes).unwrap();

        assert_eq!(bytes.len(), Tachygram::SIZE);

        let tg2 = Tachygram::zcash_deserialize(&bytes[..]).unwrap();
        assert_eq!(tg, tg2);
    }
}
