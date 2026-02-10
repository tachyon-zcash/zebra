//! Tachyon actions (Tachyactions).
//!
//! A Tachyaction is a simplified version of an Orchard Action, designed for
//! block-level proof aggregation and out-of-band payment distribution.
//!
//! Key differences from Orchard Actions:
//! - No encrypted ciphertexts (payment secrets distributed out-of-band)
//! - No nullifier or note commitment fields (those are tachygrams in the Tachystamp)
//! - Proof is aggregated at block level, not per-action
//! - Signature included directly (Sapling pattern, not Orchard wrapper pattern)

use reddsa::{orchard::SpendAuth, Signature};

use super::commitment::ValueCommitment;

/// A Tachyon action description.
///
/// Tachyactions are simpler than Orchard actions because:
/// 1. No ciphertexts - secrets distributed out-of-band
/// 2. No nullifier or cm_x - those are tachygrams in the Tachystamp
/// 3. Proofs aggregated at block level via Ragu PCD
///
/// Unlike Orchard which uses an `AuthorizedAction` wrapper type,
/// Tachyon follows the Sapling pattern where the signature is included
/// directly in the action. This is because tachyaction signatures are
/// always required (never stripped during aggregation).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tachyaction {
    /// Value commitment to net value (input - output).
    pub cv: ValueCommitment,

    /// Randomized spend authorization key.
    pub rk: reddsa::VerificationKeyBytes<SpendAuth>,

    /// The spend authorization signature.
    pub spend_auth_sig: Signature<SpendAuth>,
}

impl Tachyaction {
    /// The size of a serialized Tachyaction in bytes.
    ///
    /// cv: 32 + rk: 32 + spend_auth_sig: 64 = 128 bytes
    ///
    /// This is significantly smaller than Orchard actions (~884 bytes)
    /// due to the absence of encrypted ciphertexts and because nullifiers
    /// and note commitments are moved to the Tachystamp as tachygrams.
    pub const SIZE: usize = 128;
}
