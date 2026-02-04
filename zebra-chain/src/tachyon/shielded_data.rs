//! Tachyon shielded data for transactions.
//!
//! This module defines types for Tachyon transactions:
//!
//! - [`ShieldedData`] - Tachyon bundle containing actions, value balance, and optional tachystamp
//! - [`Tachystamp`] - Contains tachygrams, proof, and epoch (stripped during aggregation)
//!
//! ## Aggregate Transaction Model
//!
//! Tachyon uses an aggregate proof model:
//!
//! 1. Users broadcast full transactions with tachystamp (tachygrams, proof, epoch)
//! 2. Aggregators collect transactions and merge tachystamps
//! 3. In blocks, individual transactions are **stripped** (tachystamp set to None)
//! 4. The aggregate transaction (coinbase) contains the merged tachystamp
//!
//! The `tachystamp` field is `Some` when broadcast, `None` after stripping.

use std::fmt;

use halo2::pasta::pallas;
use reddsa::{orchard::Binding, Signature};

use crate::{
    amount::{Amount, NegativeAllowed},
    serialization::AtLeastOne,
};

use super::{
    accumulator, action::Tachyaction, commitment::ValueCommitment, proof::AggregateProof,
    tachygram::Tachygram,
};

/// Tachyon shielded data bundle for a transaction.
///
/// This is the main Tachyon bundle type, analogous to `sapling::ShieldedData`
/// and `orchard::ShieldedData`.
///
/// The `tachystamp` field handles both broadcast and stripped forms:
/// - `Some(tachystamp)` - Full transaction as broadcast (contains proof, tachygrams, epoch)
/// - `None` - Stripped transaction in a block (tachystamp moved to coinbase aggregate)
///
/// Note: Unlike Orchard, Tachyon does not have a `flags` field. Tachyon's unified
/// action model makes the spend/output distinction unnecessary at the protocol level.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShieldedData {
    /// Net value of Tachyon spends minus outputs.
    ///
    /// Positive means value flows out of Tachyon pool (deshielding).
    /// Negative means value flows into Tachyon pool (shielding).
    pub value_balance: Amount<NegativeAllowed>,

    /// The tachyactions (cv, rk, and spend authorization signature for each).
    ///
    /// Each action represents a spend and/or output operation.
    pub actions: AtLeastOne<Tachyaction>,

    /// Binding signature on transaction sighash.
    ///
    /// This proves that the value commitments in actions sum to the
    /// declared value_balance without revealing actual amounts.
    pub binding_sig: Signature<Binding>,

    /// The tachystamp containing proof, tachygrams, and epoch.
    ///
    /// Present when the transaction is broadcast, None after stripping
    /// during aggregation.
    pub tachystamp: Option<Tachystamp>,
}

/// Tachystamp containing the proof, tachygrams, and epoch.
///
/// This type bundles:
/// - All tachygrams (nullifiers and note commitments) for the transaction
/// - The Ragu proof proving validity of all operations
/// - The epoch (accumulator state)
///
/// During aggregation, tachystamps are merged into a single aggregate
/// tachystamp that goes into the coinbase transaction.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tachystamp {
    /// All tachygrams from this transaction.
    ///
    /// These are the nullifiers and note commitments that get recorded
    /// in the polynomial accumulator.
    pub tachygrams: Vec<Tachygram>,

    /// The Ragu proof covering all operations.
    pub proof: AggregateProof,

    /// The epoch (accumulator state).
    ///
    /// All spends in this transaction reference notes committed at or
    /// before this accumulator state.
    pub epoch: accumulator::Epoch,
}

impl Tachystamp {
    /// Create a new tachystamp.
    pub fn new(
        tachygrams: Vec<Tachygram>,
        proof: AggregateProof,
        epoch: accumulator::Epoch,
    ) -> Self {
        Self {
            tachygrams,
            proof,
            epoch,
        }
    }

    /// Get the number of tachygrams in this tachystamp.
    pub fn tachygram_count(&self) -> usize {
        self.tachygrams.len()
    }
}

impl fmt::Display for ShieldedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("tachyon::ShieldedData");
        debug
            .field("actions", &self.actions.len())
            .field("value_balance", &self.value_balance);

        if let Some(ref tachystamp) = self.tachystamp {
            debug.field(
                "tachystamp",
                &format!("{} tachygrams", tachystamp.tachygram_count()),
            );
        } else {
            debug.field("tachystamp", &"None (stripped)");
        }

        debug.finish()
    }
}

impl fmt::Display for Tachystamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("tachyon::Tachystamp")
            .field("tachygrams", &self.tachygrams.len())
            .field("proof_size", &self.proof.size())
            .finish()
    }
}

impl ShieldedData {
    /// Iterate over the actions in this bundle.
    pub fn actions(&self) -> impl Iterator<Item = &Tachyaction> {
        self.actions.iter()
    }

    /// Get the value balance of this bundle.
    pub fn value_balance(&self) -> Amount<NegativeAllowed> {
        self.value_balance
    }

    /// Calculate the binding verification key.
    ///
    /// This is used to verify the binding signature. The key is derived from
    /// the value commitments in actions and the balancing value.
    pub fn binding_verification_key(&self) -> reddsa::VerificationKeyBytes<Binding> {
        let cv: ValueCommitment = self.actions().map(|action| action.cv).sum();
        let cv_balance = ValueCommitment::new(pallas::Scalar::zero(), self.value_balance);

        let key_bytes: [u8; 32] = (cv - cv_balance).into();
        key_bytes.into()
    }

    /// Count the number of actions in this bundle.
    pub fn actions_count(&self) -> usize {
        self.actions.len()
    }

    /// Check if this transaction has been stripped (tachystamp removed).
    pub fn is_stripped(&self) -> bool {
        self.tachystamp.is_none()
    }
}
