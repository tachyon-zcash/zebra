//! Tachyon proofs.
//!
//! Tachyon uses **Ragu PCD** (Proof-Carrying Data) for proof generation and
//! aggregation. This enables efficient recursive proof composition where
//! multiple transaction proofs can be merged into a single proof.

/// Ragu proof for Tachyon transactions.
///
/// The const parameter `N` is the number of tachygrams (proof inputs)
/// this proof covers.
///
/// This is a placeholder type. The actual proof structure will be
/// defined when the Ragu PCD library is integrated.
#[derive(Clone)]
pub struct Proof<const N: usize>(pub(crate) ());

impl<const N: usize> Proof<N> {
    /// Creates a new placeholder proof.
    pub fn placeholder() -> Self {
        Self(())
    }
}
