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

use crate::primitives::Fp;
use ff::PrimeField;
pub use redpallas::{Binding, Signature, SigningKey, SpendAuth, VerificationKey};

/// Domain separator for Tachyon nullifier derivation.
///
/// Used in the GGM Tree PRF construction to domain-separate
/// nullifier computations from other hash uses.
pub const NULLIFIER_DOMAIN: &str = "z.cash:Tachyon-nf";

/// Domain separator for value commitments.
///
/// Shares Orchard's domain to reuse `reddsa::orchard::Binding` for the
/// binding signature (same generators V and R, same basepoint).
pub const VALUE_COMMITMENT_DOMAIN: &str = "z.cash:Orchard-cv";

// =============================================================================
// Spending key and child key derivation
// =============================================================================

/// A Tachyon spending key.
///
/// The root key from which all other keys are derived. This key must
/// be kept secret as it provides full spending authority.
///
/// Derives three child keys:
/// - [`SpendingKey::spend_authorizing_key`] → `ask`
/// - [`SpendingKey::nullifier_key`] → `nk`
/// - [`SpendingKey::payment_key`] → `pk`
#[derive(Clone)]
pub struct SpendingKey(Fp);

impl SpendingKey {
    /// Derives the spend authorizing key `ask` from this spending key.
    ///
    /// The returned [`SigningKey<SpendAuth>`] can be randomized per-action
    /// to produce a signing key whose corresponding verification key is
    /// the `rk` in each [`Action`](crate::Action).
    pub fn spend_authorizing_key(&self) -> SigningKey<SpendAuth> {
        // TODO: Implement PRF-based derivation (PRF_Expand with Tachyon domain)
        let bytes = self.0.to_repr();
        bytes.try_into().expect("valid signing key")
    }

    /// Derives the nullifier key `nk` from this spending key.
    ///
    /// `nk` is used in the nullifier PRF: `nf = F_nk(ψ || flavor)`.
    /// Combined with `ak`, it forms the [`ProofAuthorizingKey`] which
    /// can be delegated to an oblivious syncing service.
    pub fn nullifier_key(&self) -> NullifierKey {
        // TODO: Implement PRF-based derivation (PRF_Expand with Tachyon domain)
        NullifierKey(self.0)
    }

    /// Derives the payment key `pk` from this spending key.
    ///
    /// `pk` is used in note construction and out-of-band payment protocols.
    /// It replaces Orchard's diversified transmission key — Tachyon removes
    /// key diversification and payment addresses from the core protocol.
    pub fn payment_key(&self) -> PaymentKey {
        // TODO: Implement PRF-based derivation (PRF_Expand with Tachyon domain)
        PaymentKey(self.0)
    }
}

// =============================================================================
// Nullifier key
// =============================================================================

/// A Tachyon nullifier deriving key.
///
/// Used in the nullifier PRF: `nf = F_nk(ψ || flavor)` where the PRF
/// is a GGM tree instantiated from Poseidon. This key enables:
///
/// - **Nullifier derivation**: detecting when a note has been spent
/// - **Oblivious sync delegation**: via the master root key
///   `mk = KDF(ψ, nk)`, prefix keys `Ψ_t` can be derived that permit
///   evaluating the PRF only for epochs `e ≤ t`
///
/// `nk` alone does NOT confer spend authority — it only allows observing
/// spend status and constructing proofs (when combined with `ak`).
#[derive(Clone, Debug)]
pub struct NullifierKey(Fp);

impl NullifierKey {
    /// Returns the raw field element.
    pub fn inner(&self) -> Fp {
        self.0
    }

    /// Serializes to 32 bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_repr()
    }
}

// =============================================================================
// Payment key
// =============================================================================

/// A Tachyon payment key.
///
/// Used in note construction and out-of-band payment protocols. Replaces
/// Orchard's diversified transmission key (`pk_d`) — Tachyon removes key
/// diversification from the core protocol.
///
/// The recipient's `pk` appears in the note and is committed to in the
/// note commitment. It is NOT an on-chain address; payment coordination
/// happens out-of-band (e.g. URI encapsulated payments, payment requests).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentKey(Fp);

impl PaymentKey {
    /// Returns the raw field element.
    pub fn inner(&self) -> Fp {
        self.0
    }

    /// Serializes to 32 bytes.
    pub fn to_bytes(&self) -> [u8; 32] {
        self.0.to_repr()
    }

    /// Deserializes from 32 bytes.
    pub fn from_bytes(bytes: &[u8; 32]) -> Option<Self> {
        let elem = Fp::from_repr(*bytes);
        if elem.is_some().into() {
            Some(Self(elem.unwrap()))
        } else {
            None
        }
    }
}

// =============================================================================
// Proof authorizing key
// =============================================================================

/// The proof authorizing key (`ak` + `nk`).
///
/// Allows constructing proofs without spend authority. Can be delegated
/// to an oblivious syncing service that constructs non-membership proofs
/// for nullifiers without learning the wallet's spending key.
///
/// Derived from `ask → ak` (via the RedPallas verification key) and `nk`.
#[derive(Clone, Debug)]
pub struct ProofAuthorizingKey {
    /// The spend validating key `ak = [ask] G`.
    ak: VerificationKey<SpendAuth>,
    /// The nullifier deriving key.
    nk: NullifierKey,
}

impl ProofAuthorizingKey {
    /// Constructs a proof authorizing key from its components.
    pub fn new(ak: VerificationKey<SpendAuth>, nk: NullifierKey) -> Self {
        Self { ak, nk }
    }

    /// Derives the proof authorizing key from a spending key.
    pub fn from_spending_key(sk: &SpendingKey) -> Self {
        let ask = sk.spend_authorizing_key();
        let ak: VerificationKey<SpendAuth> = (&ask).into();
        let nk = sk.nullifier_key();
        Self { ak, nk }
    }

    /// Returns the spend validating key `ak`.
    pub fn ak(&self) -> &VerificationKey<SpendAuth> {
        &self.ak
    }

    /// Returns the nullifier deriving key `nk`.
    pub fn nk(&self) -> &NullifierKey {
        &self.nk
    }
}
