//! Tachyon proofs.
//!
//! Tachyon uses **Ragu PCD** (Proof-Carrying Data) for proof generation and
//! aggregation. This enables efficient recursive proof composition where
//! multiple transaction proofs can be merged into a single proof.

/// Ragu proof for Tachyon transactions.
///
/// This is a placeholder type. The actual proof structure will be
/// defined when the Ragu PCD library is integrated.
///
/// The proof certifies that all tachyactions in a transaction follow
/// the correct rules for spend and output operations, preserving
/// value balance integrity without revealing amounts.
#[derive(Clone, Debug)]
pub struct Proof(pub(crate) ());

impl Proof {
    /// Creates a new placeholder proof.
    pub fn placeholder() -> Self {
        Self(())
    }
}
