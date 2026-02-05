//! Tachyon shielded data for transactions.
//!
//! This module defines types for Tachyon transactions:
//!
//! - [`ShieldedData`] - Tachyon bundle containing actions, value balance, and optional tachystamp
//! - [`Tachystamp`] - Contains tachygrams, proof, and anchor (stripped during aggregation)
//!
//! ## Bundle Categories
//!
//! A Tachyon bundle falls into one of five categories based on its field values:
//!
//! | Category | actions | tachystamp | Description |
//! | -------- | ------- | ---------- | ----------- |
//! | **Adjunct** | non-empty | None | Stripped; tachystamp moved to aggregate |
//! | **Autonome** | non-empty | Some (len = actions) | Self-contained with own tachystamp |
//! | **Aggregate** | any | Some (len > actions) | Carries merged tachystamp for adjuncts |
//!
//! Aggregates have two flavors:
//! - **Based** (`!actions.is_empty()`): has own actions; "based" = coin*base*
//! - **Innocent** (`actions.is_empty()`): only carries others' tachystamps
//!
//! Note: When both `actions` is empty AND `tachystamp` is None, the bundle serializes
//! as `Option<ShieldedData>::None` (no Tachyon activity).
//!
//! ## Aggregate Transaction Model
//!
//! Tachyon uses an aggregate proof model:
//!
//! 1. Users broadcast **autonome** transactions with complete tachystamp
//! 2. Aggregators collect transactions and merge tachystamps
//! 3. In blocks, individual transactions become **adjuncts** (tachystamp set to None)
//! 4. The **based aggregate** (typically coinbase) contains merged tachystamp + own actions
//! 5. An **innocent aggregate** carries only merged tachystamps with no own actions

use std::fmt;

use halo2::pasta::pallas;
use reddsa::{orchard::Binding, Signature};

use crate::amount::{Amount, NegativeAllowed};

use super::{
    accumulator, action::Tachyaction, commitment::ValueCommitment, proof::Proof,
    tachygram::Tachygram,
};

/// Tachyon shielded data bundle for a transaction.
///
/// This is the main Tachyon bundle type, analogous to `sapling::ShieldedData`
/// and `orchard::ShieldedData`.
///
/// ## Bundle Categories
///
/// The combination of `actions` and `tachystamp` determines the bundle category:
///
/// - **Adjunct**: `tachystamp.is_none() && !actions.is_empty()`
///   - Stripped transaction; tachystamp moved to an aggregate
///   - Depends on a preceding aggregate in the block for proof verification
///
/// - **Autonome**: `tachystamp.is_some() && tachygrams.len() == actions.len()`
///   - Self-contained transaction with its own complete tachystamp
///   - Can appear anywhere in a block without disrupting aggregate-adjunct sequences
///
/// - **Aggregate**: `tachystamp.is_some() && tachygrams.len() > actions.len()`
///   - Carries merged tachystamp covering following adjuncts
///   - **Based aggregate**: has own actions; "based" = coin*base* (miner shielding rewards)
///   - **Innocent aggregate**: no own actions, only carries others' tachystamps
///
/// Note: Unlike Orchard, Tachyon does not have a `flags` field. Tachyon's unified
/// action model makes the spend/output distinction unnecessary at the protocol level.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShieldedData {
    /// The tachyactions (cv, rk, and spend authorization signature for each).
    ///
    /// Each action represents a spend and/or output operation.
    /// Empty for pure aggregates that only carry others' tachystamps.
    pub actions: Vec<Tachyaction>,

    /// Net value of Tachyon spends minus outputs.
    ///
    /// Positive means value flows out of Tachyon pool (deshielding).
    /// Negative means value flows into Tachyon pool (shielding).
    /// Zero when actions is empty.
    pub value_balance: Amount<NegativeAllowed>,

    /// Binding signature on transaction sighash.
    ///
    /// This proves that the value commitments in actions sum to the
    /// declared value_balance without revealing actual amounts.
    /// None when actions is empty (nothing to sign over).
    pub binding_sig: Option<Signature<Binding>>,

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
    pub proof: Proof,

    /// The epoch (accumulator state).
    ///
    /// All spends in this transaction reference notes committed at or
    /// before this accumulator state.
    pub anchor: accumulator::Epoch,
}

impl Tachystamp {
    /// Create a new tachystamp.
    pub fn new(tachygrams: Vec<Tachygram>, proof: Proof, anchor: accumulator::Epoch) -> Self {
        Self {
            tachygrams,
            proof,
            anchor,
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
            .field("proof_size", &self.proof.as_bytes().len())
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
    ///
    /// Returns None if there are no actions (nothing to verify).
    pub fn binding_verification_key(&self) -> Option<reddsa::VerificationKeyBytes<Binding>> {
        if self.actions.is_empty() {
            return None;
        }

        let cv: ValueCommitment = self.actions().map(|action| action.cv).sum();
        let cv_balance = ValueCommitment::new(pallas::Scalar::zero(), self.value_balance);

        let key_bytes: [u8; 32] = (cv - cv_balance).into();
        Some(key_bytes.into())
    }

    /// Count the number of actions in this bundle.
    pub fn actions_count(&self) -> usize {
        self.actions.len()
    }

    /// Check if this is an adjunct bundle (stripped, tachystamp moved to aggregate).
    ///
    /// An adjunct has actions but no tachystamp. It depends on a preceding
    /// aggregate transaction in the block for proof verification.
    pub fn is_adjunct(&self) -> bool {
        !self.actions.is_empty() && self.tachystamp.is_none()
    }

    /// Check if this is an autonome bundle (self-contained with own tachystamp).
    ///
    /// An autonome has actions and a tachystamp where the tachygram count
    /// equals the action count (one tachygram per action).
    pub fn is_autonome(&self) -> bool {
        if let Some(ref stamp) = self.tachystamp {
            !self.actions.is_empty() && stamp.tachygrams.len() == self.actions.len()
        } else {
            false
        }
    }

    /// Check if this is an aggregate bundle (carries merged tachystamp for adjuncts).
    ///
    /// An aggregate has a tachystamp where the tachygram count exceeds the
    /// action count, meaning it carries tachygrams from following adjuncts.
    ///
    /// Aggregates come in two flavors:
    /// - **Based**: has own actions ("based" = coin*base*, miner shielding rewards)
    /// - **Innocent**: no own actions, only carries others' tachystamps
    pub fn is_aggregate(&self) -> bool {
        if let Some(ref stamp) = self.tachystamp {
            stamp.tachygrams.len() > self.actions.len()
        } else {
            false
        }
    }

    /// Check if this is an innocent aggregate (no own actions).
    ///
    /// An innocent aggregate has no actions but has a tachystamp containing
    /// merged tachygrams from other transactions. Contrast with a "based"
    /// aggregate which has its own actions (typically coinbase shielding rewards).
    pub fn is_innocent_aggregate(&self) -> bool {
        self.actions.is_empty() && self.tachystamp.is_some()
    }

    /// Check if this transaction has been stripped (tachystamp removed).
    ///
    /// Alias for [`is_adjunct`](Self::is_adjunct) - a stripped transaction
    /// is one whose tachystamp has been moved to an aggregate.
    pub fn is_stripped(&self) -> bool {
        self.is_adjunct()
    }
}
