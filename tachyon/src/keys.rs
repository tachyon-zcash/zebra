//! Tachyon key types.
//!
//! ## Key Hierarchy
//!
//! ```text
//! SpendingKey (sk)
//!     │
//!     ├── NullifierKey (nk)
//!     │
//!     └── FullViewingKey (fvk)
//!              │
//!              └── IncomingViewingKey (ivk)
//! ```

use ff::Field;

use crate::note::{Epoch, Nullifier, NullifierTrapdoor};
use crate::primitives::Fp;

/// A Tachyon spending key.
///
/// The root key from which all other keys are derived. This key must
/// be kept secret as it provides full spending authority.
#[derive(Clone, Debug)]
pub struct SpendingKey(Fp);

impl SpendingKey {
    /// Derives the nullifier key from this spending key.
    pub fn nullifier_key(&self) -> NullifierKey {
        // TODO: Implement proper key derivation using Poseidon
        NullifierKey(self.0)
    }

    /// Derives the full viewing key from this spending key.
    pub fn full_viewing_key(&self) -> FullViewingKey {
        // TODO: Implement proper key derivation
        FullViewingKey(self.0)
    }
}

/// A Tachyon full viewing key.
///
/// Allows viewing all incoming and outgoing transactions but cannot
/// spend funds.
#[derive(Clone, Debug)]
pub struct FullViewingKey(Fp);

impl FullViewingKey {
    /// Derives the incoming viewing key from this full viewing key.
    pub fn incoming_viewing_key(&self) -> IncomingViewingKey {
        // TODO: Implement proper key derivation
        IncomingViewingKey(self.0)
    }
}

/// A Tachyon incoming viewing key.
///
/// Allows viewing incoming transactions only. Cannot see outgoing
/// transactions or spend funds.
#[derive(Clone, Debug)]
pub struct IncomingViewingKey(Fp);

/// A Tachyon nullifier key $\mathsf{nk}$.
///
/// Used for nullifier derivation. The nullifier for a note is computed as:
///
/// $$\mathsf{nf} = F_{\mathsf{nk}}(\Psi \| e)$$
///
/// where $\Psi$ is the nullifier trapdoor and $e$ is the epoch.
#[derive(Clone, Debug)]
pub struct NullifierKey(Fp);

impl NullifierKey {
    /// Derives a nullifier from this key, a trapdoor, and an epoch.
    ///
    /// This implements the simplified Tachyon nullifier formula:
    /// $\mathsf{nf} = F_{\mathsf{nk}}(\Psi \| e)$
    pub fn derive_nullifier(&self, trapdoor: &NullifierTrapdoor, epoch: Epoch) -> Nullifier {
        // TODO: Implement actual Poseidon-based PRF
        // For now, return a placeholder combining the inputs
        let _ = (self.0, trapdoor.0, epoch.0);
        Nullifier(Fp::ZERO)
    }
}
