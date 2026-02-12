//! Tachyon proofs.
//!
//! Tachyon uses **Ragu PCD** (Proof-Carrying Data) for proof generation and
//! aggregation. A single Ragu proof per aggregate covers all actions across
//! multiple bundles.
//!
//! ## Verification
//!
//! The verifier receives accumulated public inputs — not matched pairs:
//!
//! - **Actions** (`Vec<Action>`) — from bundles, with `cv`, `rk`, `sig`
//! - **Tachygrams** (`Vec<Tachygram>`) — from the stamp, accumulated
//! - **Anchor** — the accumulator state reference (epoch)
//!
//! The proof asserts that each action's `rk` and value commitment are
//! consistent with the corresponding tachygram.
//!
//! ## Proving
//!
//! The prover supplies an [`ActionWitness`] per action, containing private
//! inputs that the circuit checks against the public action and tachygram.

use crate::action::{Action, ValueCommitTrapdoor};
use crate::note::Note;
use crate::primitives::{Epoch, Fp, Fq, Tachygram};

/// Ragu proof for Tachyon transactions.
///
/// Opaque byte blob covering all actions in an aggregate. The internal
/// structure will be defined by the Ragu PCD library; methods on this
/// type are stubs marking the design boundary.
#[derive(Clone, Debug)]
pub struct Proof(Vec<u8>);

impl Proof {
    /// Returns the raw proof bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Constructs a proof from raw bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Proof(bytes)
    }
}

impl Default for Proof {
    fn default() -> Self {
        Proof(vec![0u8; 192]) // placeholder length
    }
}

/// An error returned when proof verification fails.
pub enum ProofValidationError {
    /// The proof did not verify.
    Failure,
}

impl Proof {
    /// Creates a proof from action witnesses.
    ///
    /// Each witness carries a token (nf/cmx); Ragu PCD rerandomizes
    /// them into the published tachygrams returned alongside the proof.
    pub fn create(
        _witnesses: &Vec<ActionWitness>,
        _actions: &Vec<Action>,
        _anchor: &Epoch,
    ) -> (Self, Vec<Tachygram>) {
        // TODO: Ragu PCD — tokens from witnesses become
        // rerandomized tachygrams in the output
        (Proof(Vec::new()), Vec::new())
    }

    /// Merges two proofs (Ragu PCD fuse).
    ///
    /// Used during aggregation to combine stamps from multiple bundles.
    pub fn merge(_left: Self, _right: Self) -> Self {
        todo!("Ragu PCD fuse")
    }

    /// Verifies this proof against accumulated public inputs.
    ///
    /// `actions` and `tachygrams` are collected independently during
    /// aggregation — the proof is what binds them together.
    pub fn verify(
        &self,
        _actions: Vec<Action>,
        _tachygrams: Vec<Tachygram>,
        _anchor: Epoch,
    ) -> Result<(), ProofValidationError> {
        // TODO: Ragu verification
        Ok(())
    }
}

// =============================================================================
// Action witnesses (prover-side)
// =============================================================================

/// Private witness for a single action.
pub struct ActionWitness {
    /// The note being spent or created.
    pub note: Note,

    /// Signed value: positive for spends, negative for outputs.
    pub value: i64,

    /// Randomizer for the verification key.
    ///
    /// Spend: `rk = ak + [α]G`.  Output: `rk = [α]G`.
    pub alpha: Fq,

    /// Value commitment randomness.
    pub rcv: ValueCommitTrapdoor,

    /// Epoch anchor for this action.
    pub flavor: Epoch,

    /// The derived token (nullifier or note commitment) as a field element.
    /// Rerandomized into a tachygram by the prover.
    pub token: Fp,
}
