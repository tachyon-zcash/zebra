//! Tachyon proofs.
//!
//! Tachyon uses **Ragu PCD** (Proof-Carrying Data) for proof generation and
//! aggregation. This enables efficient recursive proof composition where
//! multiple transaction proofs can be merged into a single proof.
//!
//! ## Wire Format
//!
//! | Field | Size | Description |
//! |-------|------|-------------|
//! | size | compactsize | Length of proof data |
//! | data | size bytes | Opaque proof bytes |

/// Ragu proof for Tachyon transactions.
///
/// This wraps [`tachyon::Proof`] with serialization support.
///
/// The proof certifies that all tachyactions in a transaction follow
/// the correct rules for spend and output operations, preserving
/// value balance integrity without revealing amounts.
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
        // Placeholder: return minimal valid proof
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
