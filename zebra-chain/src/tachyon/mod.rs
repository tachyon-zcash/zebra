use group::GroupEncoding;
use halo2::pasta::pallas;
use serde::{Deserialize, Serialize};

use crate::serialization::serde_helpers;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ShieldedTransactionAggregate {}

/// A tachygram, represented as a Pallas curve point (similar to Orchard note commitments).
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct Tachygram(#[serde(with = "serde_helpers::Affine")] pub pallas::Affine);

impl std::fmt::Debug for Tachygram {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("Tachygram")
            .field(&hex::encode(self.0.to_bytes()))
            .finish()
    }
}

impl Tachygram {
    /// Extract the x-coordinate of the tachygram point for use in merkle trees.
    pub fn extract_x(&self) -> pallas::Base {
        crate::orchard::sinsemilla::extract_p(self.0.into())
    }
}

#[cfg(any(test, feature = "proptest-impl"))]
pub mod arbitrary;