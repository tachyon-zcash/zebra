//! Tachyon proofs.
//!
//! Tachyon uses Ragu PCD for proof generation and aggregation.

/// Ragu proof for Tachyon transactions.
///
/// This is a placeholder type. The actual proof structure will be
/// defined when the Ragu PCD library is integrated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proof(pub(crate) ());

impl Proof {
    /// Creates a new placeholder proof.
    pub fn placeholder() -> Self {
        Self(())
    }
}
