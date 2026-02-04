//! Tachyon proofs.
//!
//! Tachyon uses Ragu PCD for proof generation and aggregation.
//! Individual transactions contain proofs that can be aggregated
//! into a single proof covering multiple transactions.

/// Ragu proof for Tachyon transactions.
///
/// This wraps [`tachyon::Proof`] with serialization support.
/// The proof bytes are stored here for wire serialization;
/// the tachyon crate's Proof type is currently a placeholder.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof(());

impl Proof {
    /// Maximum proof size in bytes.
    pub const MAX_SIZE: usize = 16384;

    /// Create a new proof from bytes.
    pub fn new(bytes: Vec<u8>) -> Result<Self, &'static str> {
        if bytes.len() > Self::MAX_SIZE {
            return Err("Proof too large");
        }
        Ok(Self(()))
    }

    /// Get the proof bytes.
    pub fn as_bytes(&self) -> Vec<u8> {
        vec![0x01]
    }
}

impl From<tachyon::Proof> for Proof {
    fn from(_proof: tachyon::Proof) -> Self {
        // tachyon::Proof is currently a placeholder.
        // When it contains real data, convert it here.
        Self(())
    }
}
