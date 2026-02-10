//! ## Key Hierarchy
//!
//! Tachyon simplifies the key hierarchy compared to Orchard by removing
//! key diversification, viewing keys, and payment addresses from the core
//! protocol. These capabilities are handled by higher-level wallet software
//! through out-of-band payment protocols.
//!
//! ```mermaid
//! flowchart TB
//!     sk[SpendingKey sk]
//!     ask[ask SigningKey SpendAuth]
//!     nk[NullifierKey nk]
//!     pk[PaymentKey pk]
//!     ak[ak VerificationKey SpendAuth]
//!     pak[ProofAuthorizingKey]
//!     sk --> ask
//!     sk --> nk
//!     sk --> pk
//!     ask --> ak
//!     ak --> pak
//!     nk --> pak
//! ```
//!
//! - **ask**: Authorizes spends (RedPallas signing key)
//! - **ak + nk** (proof authorizing key): Constructs proofs without spend
//!   authority; can be delegated to an oblivious syncing service
//! - **nk**: Observes when funds are spent (nullifier derivation)
//! - **pk**: Used in note construction and out-of-band payment protocols
//!
//! ## Nullifier Derivation
//!
//! Nullifiers are derived via a GGM tree PRF instantiated from Poseidon:
//!
//! $$\mathsf{nf} = F_{\mathsf{nk}}(\Psi \parallel \tau)$$
//!
//! where $\Psi$ is the nullifier trapdoor and $\tau$ is the epoch.
//!
//! The master root key $\mathsf{mk} = \text{KDF}(\Psi, \mathsf{nk})$ supports
//! oblivious sync delegation: prefix keys $\Psi_t$ permit evaluating the PRF
//! only for epochs $e \leq t$, enabling range-restricted delegation without
//! revealing spend capability.

mod redpallas;

use crate::primitives::{Fp, Fq, Tachygram};
use ff::{Field, PrimeField};
pub use redpallas::{Binding, Signature, SigningKey, SpendAuth, VerificationKey};

/// Domain separator for Tachyon nullifier derivation.
///
/// Used in the GGM Tree PRF construction to domain-separate
/// nullifier computations from other hash uses.
pub const NULLIFIER_DOMAIN: &str = "Tachyon_Nullifier";

/// Domain separator for Tachyon note commitments.
pub const NOTE_COMMITMENT_DOMAIN: &str = "Tachyon_NoteCommit";

/// Domain separator for Tachyon value commitments.
pub const VALUE_COMMITMENT_DOMAIN: &str = "Tachyon_ValueCommit";

/// Domain separator for the polynomial accumulator.
pub const ACCUMULATOR_DOMAIN: &str = "Tachyon_Accumulator";

/// A Tachyon spending key.
///
/// The root key from which all other keys are derived. This key must
/// be kept secret as it provides full spending authority.
#[derive(Clone)]
pub struct SpendingKey(Fp);

impl SpendingKey {
    /// Derives the spend authorizing key `ask` from this spending key.
    ///
    /// The returned [`SigningKey<SpendAuth>`] can be randomized per-action
    /// to produce a signing key whose corresponding verification key is
    /// the `rk` in each [`Action`](crate::Action).
    pub fn spend_authorizing_key(&self) -> SigningKey<SpendAuth> {
        // TODO: Implement proper PRF-based derivation
        let bytes = self.0.to_repr();
        bytes.try_into().expect("valid signing key")
    }

    /// Derives the nullifier key from this spending key.
    pub fn nullifier_key(&self) -> NullifierKey {
        // TODO: Implement proper key derivation using Poseidon
        NullifierKey(self.0)
    }

    /// Derives the payment key from this spending key.
    pub fn payment_key(&self) -> PaymentKey {
        // TODO: Implement proper key derivation
        PaymentKey(self.0)
    }
}

/// A Tachyon nullifier key.
///
/// Used for nullifier derivation via a GGM tree PRF instantiated from
/// Poseidon.
#[derive(Clone)]
pub struct NullifierKey(Fp);

impl NullifierKey {
    /// Derives a nullifier tachygram from this key, a trapdoor, and an epoch.
    pub fn derive_tachygram(&self, _trapdoor: &Fq, _epoch: crate::primitives::Epoch) -> Tachygram {
        // TODO: Implement actual Poseidon-based GGM tree PRF
        Tachygram(Fp::ZERO)
    }
}

/// A Tachyon payment key.
///
/// Used in note construction and out-of-band payment protocols.
#[derive(Clone)]
pub struct PaymentKey(Fp);
