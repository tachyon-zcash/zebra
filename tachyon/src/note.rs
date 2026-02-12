//! Tachyon notes and note commitments.
//!
//! A Tachyon note is simpler than an Orchard note: no diversifier, no `rho`,
//! no unique value for faerie gold defense. Out-of-band payment protocols
//! handle payment coordination, and the nullifier construction doesn't
//! require global uniqueness.
//!
//! ## Note Structure
//!
//! | Field | Type | Description |
//! | ----- | ---- | ----------- |
//! | `pk`  | [`PaymentKey`] | Recipient's payment key |
//! | `v`   | `u64` | Note value |
//! | `psi` | `Fp`  | Nullifier trapdoor (ψ) |
//! | `rcm` | `Fq`  | Note commitment randomness |
//!
//! Both `ψ` and `rcm` can be derived from a shared key negotiated
//! through the out-of-band payment protocol.
//!
//! ## Nullifier Derivation
//!
//! `nf = F_nk(ψ || flavor)` where `F` is a GGM tree PRF from Poseidon.
//! This is evaluated in-circuit by the Ragu proof system.
//!
//! ## Note Commitment
//!
//! A commitment over the note fields, producing a `cmx` tachygram that
//! enters the polynomial accumulator. The concrete commitment scheme
//! (e.g. Sinsemilla, Poseidon) depends on what is efficient inside
//! Ragu circuits and is TBD.

use crate::keys::{NullifierKey, PaymentKey};
use crate::primitives::{Epoch, Fp, Fq};

/// A Tachyon note.
///
/// Represents a discrete unit of value in the Tachyon shielded pool.
/// Created by output operations, consumed by spend operations.
#[derive(Clone, Debug)]
pub struct Note {
    /// The recipient's payment key.
    pub pk: PaymentKey,

    /// The note value in zatoshis.
    pub value: u64,

    /// The nullifier trapdoor (ψ).
    ///
    /// Used in nullifier derivation: `nf = F_nk(ψ || flavor)`.
    /// The master root key `mk = KDF(ψ, nk)` enables oblivious
    /// sync delegation via GGM tree prefix keys.
    pub psi: Fp,

    /// Note commitment randomness.
    ///
    /// Blinds the note commitment. Can be derived from a shared
    /// secret negotiated out-of-band.
    pub rcm: Fq,
}

impl Note {
    /// Computes the note commitment `cmx`.
    ///
    /// Commits to `(pk, v, ψ)` with randomness `rcm`
    pub fn commitment(&self) -> NoteCommitment {
        // TODO: Implement note commitment
        //   cmx = NoteCommit_rcm("z.cash:Tachyon-NoteCommit", pk || v || ψ)
        todo!("note commitment")
    }

    /// Derives a nullifier for this note.
    ///
    /// The `flavor` parameter is the epoch at which the nullifier
    /// is revealed, enabling range-restricted delegation.
    pub fn nullifier(&self, _nk: &NullifierKey, _flavor: Epoch) -> Nullifier {
        // TODO: Implement Poseidon GGM tree PRF
        //   nf = F_nk(ψ || flavor)
        todo!("Poseidon GGM tree PRF nullifier derivation")
    }
}

// =============================================================================
// Note commitment (cmx)
// =============================================================================

/// A Tachyon note commitment (`cmx`).
///
/// A field element produced by committing to the note fields. This is
/// the value that becomes a tachygram:
/// - For **output** operations, `cmx` IS the tachygram directly.
/// - For **spend** operations, `cmx` is a private witness; the
///   tachygram is the derived nullifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NoteCommitment(pub Fp);

// =============================================================================
// Nullifier
// =============================================================================

/// A Tachyon nullifier.
///
/// Derived as `nf = F_nk(ψ || flavor)` where `F` is a GGM tree PRF
/// instantiated from Poseidon. Published when a note is spent; becomes
/// a tachygram in the polynomial accumulator.
///
/// Unlike Orchard, Tachyon nullifiers:
/// - Don't need collision resistance (no faerie gold defense)
/// - Have an epoch "flavor" component for oblivious sync
/// - Are prunable by validators after a window of blocks
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Nullifier(pub Fp);
