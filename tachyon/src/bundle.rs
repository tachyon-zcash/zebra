//! Tachyon transaction bundles and authorization traits.
//!
//! This module defines the bundle types for Tachyon transactions:
//!
//! - [`Bundle`] - The main bundle type containing actions and authorization
//! - [`Tachystamp`] - Contains tachygrams, proof, and epoch
//! - [`Authorization`] - Trait for tracking bundle authorization state
//!
//! ## Authorization States
//!
//! Bundles progress through authorization states using type-state pattern:
//!
//! - [`Unsigned`] - Actions created but not signed
//! - [`Autonome`] - Self-contained: signed with tachystamp (can stand alone)
//! - [`Adjunct`] - Dependent: signed but tachystamp removed (depends on aggregate)
//! - [`Aggregate`] - Merged tachystamp covering multiple adjunct bundles
//!
//! ## Block Structure
//!
//! A block can contain:
//! - `Bundle<Autonome, V>` - Standalone transactions with their own proof
//! - `Bundle<Adjunct, V>` - Dependent transactions (proof in aggregate)
//! - `Bundle<Aggregate, V>` - Aggregate transaction(s) covering adjunct bundles
//!
//! Multiple aggregates can exist in one block, each covering a different set of
//! adjunct bundles.

use std::fmt;
use std::vec::Vec;

use crate::Action;
use crate::Tachygram;
use crate::action::{SpendAuthSignature, Unsigned};
use crate::note::Epoch;

/// A binding signature for value balance verification.
///
/// This signature proves that the value commitments in actions sum to the
/// declared value balance without revealing actual amounts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindingSignature(pub [u8; 64]);

impl From<[u8; 64]> for BindingSignature {
    fn from(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }
}

impl From<BindingSignature> for [u8; 64] {
    fn from(sig: BindingSignature) -> Self {
        sig.0
    }
}

/// A placeholder for the Ragu proof.
///
/// This will be replaced with the actual Ragu proof type when available.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Proof(Vec<u8>);

impl Proof {
    /// Creates a new proof from bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Returns the byte representation of this proof.
    pub fn to_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Creates an empty proof (for testing).
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    /// Returns the size of this proof in bytes.
    pub fn size(&self) -> usize {
        self.0.len()
    }
}

/// Tachystamp containing the proof, tachygrams, and epoch.
///
/// This type bundles:
/// - All tachygrams (nullifiers and note commitments) for the transaction
/// - The Ragu proof proving validity of all operations
/// - The epoch (accumulator anchor)
///
/// Present in [`Autonome`] bundles (self-contained) and [`Aggregate`] bundles
/// (covering multiple [`Adjunct`] bundles).
///
/// Epochs from multiple tachystamps can be accumulated into a single epoch
/// during proof aggregation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tachystamp {
    /// All tachygrams from this transaction.
    ///
    /// These are the nullifiers and note commitments that get recorded
    /// in the polynomial accumulator.
    tachygrams: Vec<Tachygram>,

    /// The Ragu proof covering all operations.
    proof: Proof,

    /// The epoch (recent accumulator state).
    ///
    /// All spends in this transaction reference notes committed at or
    /// before this accumulator state. Epochs are valid within a range.
    epoch: Epoch,
}

impl Tachystamp {
    /// Creates a new tachystamp.
    pub fn new(tachygrams: Vec<Tachygram>, proof: Proof, epoch: Epoch) -> Self {
        Self {
            tachygrams,
            proof,
            epoch,
        }
    }

    /// Returns the tachygrams in this tachystamp.
    pub fn tachygrams(&self) -> &[Tachygram] {
        &self.tachygrams
    }

    /// Returns the proof.
    pub fn proof(&self) -> &Proof {
        &self.proof
    }

    /// Returns the epoch.
    pub fn epoch(&self) -> &Epoch {
        &self.epoch
    }

    /// Returns the number of tachygrams in this tachystamp.
    pub fn tachygram_count(&self) -> usize {
        self.tachygrams.len()
    }
}

// =============================================================================
// Authorization trait and states
// =============================================================================

/// Marker trait for bundle authorization states.
///
/// This trait enables type-state pattern for tracking bundle progress
/// through the signing and aggregation workflow.
///
/// ## States
///
/// - [`Unsigned`] - Bundle being constructed, no signatures
/// - [`Autonome`] - Self-contained bundle with signatures and tachystamp
/// - [`Adjunct`] - Dependent bundle with signatures but no tachystamp
/// - [`Aggregate`] - Merged tachystamp covering adjunct bundles
pub trait Authorization: fmt::Debug {
    /// The type of authorization data stored in each action.
    type SpendAuth: fmt::Debug;
}

/// Authorization state for unsigned bundles.
///
/// Used during bundle construction before signatures are applied.
impl Authorization for Unsigned {
    type SpendAuth = Unsigned;
}

/// Authorization state for self-contained bundles.
///
/// An autonome bundle has everything needed to validate independently:
/// - Signed actions with spend authorization signatures
/// - Binding signature
/// - Tachystamp (tachygrams, proof, anchor)
///
/// Autonome bundles can appear directly in blocks without an aggregate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Autonome {
    /// Binding signature on transaction sighash.
    binding_sig: BindingSignature,

    /// The tachystamp containing proof, tachygrams, and anchor.
    tachystamp: Tachystamp,
}

impl Autonome {
    /// Creates a new autonome authorization state.
    pub fn new(binding_sig: BindingSignature, tachystamp: Tachystamp) -> Self {
        Self {
            binding_sig,
            tachystamp,
        }
    }

    /// Returns the binding signature.
    pub fn binding_sig(&self) -> &BindingSignature {
        &self.binding_sig
    }

    /// Returns the tachystamp.
    pub fn tachystamp(&self) -> &Tachystamp {
        &self.tachystamp
    }
}

impl Authorization for Autonome {
    type SpendAuth = SpendAuthSignature;
}

/// Authorization state for adjunct (dependent) bundles.
///
/// An adjunct bundle has been stripped of its tachystamp, which was
/// contributed to an [`Aggregate`]. It contains:
/// - Signed actions with spend authorization signatures
/// - Binding signature
/// - NO tachystamp (covered by aggregate)
///
/// Adjunct bundles require a corresponding aggregate in the same block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Adjunct {
    /// Binding signature on transaction sighash.
    binding_sig: BindingSignature,
}

impl Adjunct {
    /// Creates a new adjunct authorization state.
    pub fn new(binding_sig: BindingSignature) -> Self {
        Self { binding_sig }
    }

    /// Returns the binding signature.
    pub fn binding_sig(&self) -> &BindingSignature {
        &self.binding_sig
    }
}

impl Authorization for Adjunct {
    type SpendAuth = SpendAuthSignature;
}

/// Authorization state for aggregate bundles.
///
/// An aggregate bundle contains merged proof data covering multiple
/// [`Adjunct`] bundles:
/// - Merged tachystamp (combined tachygrams, aggregated proof, anchor)
/// - Binding signature
/// - Optional actions (e.g., miner fee outputs to Tachyon)
///
/// Multiple aggregates can exist in a single block, each covering a
/// different set of adjunct bundles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Aggregate {
    /// Binding signature on transaction sighash.
    binding_sig: BindingSignature,

    /// The merged tachystamp covering multiple adjunct bundles.
    tachystamp: Tachystamp,
}

impl Aggregate {
    /// Creates a new aggregate authorization state.
    pub fn new(binding_sig: BindingSignature, tachystamp: Tachystamp) -> Self {
        Self {
            binding_sig,
            tachystamp,
        }
    }

    /// Returns the binding signature.
    pub fn binding_sig(&self) -> &BindingSignature {
        &self.binding_sig
    }

    /// Returns the merged tachystamp.
    pub fn tachystamp(&self) -> &Tachystamp {
        &self.tachystamp
    }
}

impl Authorization for Aggregate {
    type SpendAuth = SpendAuthSignature;
}

// =============================================================================
// Bundle type
// =============================================================================

/// A bundle of Tachyon [`Action`] descriptions and authorization data.
///
/// This is the main Tachyon bundle type, analogous to Orchard's bundle.
///
/// ## Type Parameters
///
/// - `A`: The authorization state ([`Unsigned`], [`Autonome`], [`Adjunct`], [`Aggregate`])
/// - `V`: The value balance type (e.g., `i64` or a currency-specific type)
///
/// ## Authorization Workflow
///
/// ```text
/// Bundle<Unsigned, V>  ──sign()──►  Bundle<Autonome, V>  ──adjoin()──►  Bundle<Adjunct, V>
///                                          │                                     │
///                                          ▼                                     ▼
///                                   (valid in block)                  (needs Bundle<Aggregate, V>)
/// ```
///
/// ## Note
///
/// Unlike Orchard, Tachyon does not have a `flags` field. Tachyon's unified
/// action model makes the spend/output distinction unnecessary at the protocol level.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bundle<A: Authorization, V> {
    /// Net value of Tachyon spends minus outputs.
    ///
    /// Positive means value flows out of Tachyon pool (deshielding).
    /// Negative means value flows into Tachyon pool (shielding).
    value_balance: V,

    /// The tachyactions (cv, rk, and authorization for each).
    ///
    /// Each action represents a spend and/or output operation.
    actions: Vec<Action<A::SpendAuth>>,

    /// Bundle-level authorization data.
    authorization: A,
}

// -----------------------------------------------------------------------------
// Unsigned bundle
// -----------------------------------------------------------------------------

impl<V> Bundle<Unsigned, V> {
    /// Creates a new unsigned bundle.
    pub fn new(value_balance: V, actions: Vec<Action<Unsigned>>) -> Self {
        Self {
            value_balance,
            actions,
            authorization: Unsigned,
        }
    }

    /// Signs this bundle to produce an autonome (self-contained) bundle.
    ///
    /// # Arguments
    ///
    /// * `sign_action` - Function to sign each action, given its index
    /// * `binding_sig` - The binding signature for the bundle
    /// * `tachystamp` - The tachystamp (tachygrams, proof, anchor)
    pub fn sign<E>(
        self,
        mut sign_action: impl FnMut(usize, Action<Unsigned>) -> Result<Action<SpendAuthSignature>, E>,
        binding_sig: BindingSignature,
        tachystamp: Tachystamp,
    ) -> Result<Bundle<Autonome, V>, E> {
        let actions = self
            .actions
            .into_iter()
            .enumerate()
            .map(|(i, action)| sign_action(i, action))
            .collect::<Result<Vec<_>, E>>()?;

        Ok(Bundle {
            value_balance: self.value_balance,
            actions,
            authorization: Autonome::new(binding_sig, tachystamp),
        })
    }
}

// -----------------------------------------------------------------------------
// Autonome bundle
// -----------------------------------------------------------------------------

impl<V> Bundle<Autonome, V> {
    /// Creates a new autonome bundle directly.
    ///
    /// This is primarily for deserialization; prefer using
    /// [`Bundle::new`] and [`Bundle::sign`] for construction.
    pub fn from_parts(
        value_balance: V,
        actions: Vec<Action<SpendAuthSignature>>,
        binding_sig: BindingSignature,
        tachystamp: Tachystamp,
    ) -> Self {
        Self {
            value_balance,
            actions,
            authorization: Autonome::new(binding_sig, tachystamp),
        }
    }

    /// Returns the binding signature.
    pub fn binding_sig(&self) -> &BindingSignature {
        self.authorization.binding_sig()
    }

    /// Returns the tachystamp.
    pub fn tachystamp(&self) -> &Tachystamp {
        self.authorization.tachystamp()
    }

    /// Converts this bundle to an adjunct by removing its tachystamp.
    ///
    /// The tachystamp should be contributed to an [`Aggregate`].
    /// Returns the adjunct bundle and the extracted tachystamp.
    pub fn adjoin(self) -> (Bundle<Adjunct, V>, Tachystamp) {
        let tachystamp = self.authorization.tachystamp;
        let binding_sig = self.authorization.binding_sig;

        let bundle = Bundle {
            value_balance: self.value_balance,
            actions: self.actions,
            authorization: Adjunct::new(binding_sig),
        };

        (bundle, tachystamp)
    }
}

// -----------------------------------------------------------------------------
// Adjunct bundle
// -----------------------------------------------------------------------------

impl<V> Bundle<Adjunct, V> {
    /// Creates a new adjunct bundle directly.
    ///
    /// This is primarily for deserialization.
    pub fn from_parts(
        value_balance: V,
        actions: Vec<Action<SpendAuthSignature>>,
        binding_sig: BindingSignature,
    ) -> Self {
        Self {
            value_balance,
            actions,
            authorization: Adjunct::new(binding_sig),
        }
    }

    /// Returns the binding signature.
    pub fn binding_sig(&self) -> &BindingSignature {
        self.authorization.binding_sig()
    }
}

// -----------------------------------------------------------------------------
// Aggregate bundle
// -----------------------------------------------------------------------------

impl<V> Bundle<Aggregate, V> {
    /// Creates a new aggregate bundle.
    ///
    /// An aggregate bundle contains:
    /// - Merged tachystamp covering adjunct bundles
    /// - Optional actions (e.g., miner fee outputs)
    /// - Its own value balance and binding signature
    pub fn from_parts(
        value_balance: V,
        actions: Vec<Action<SpendAuthSignature>>,
        binding_sig: BindingSignature,
        tachystamp: Tachystamp,
    ) -> Self {
        Self {
            value_balance,
            actions,
            authorization: Aggregate::new(binding_sig, tachystamp),
        }
    }

    /// Returns the binding signature.
    pub fn binding_sig(&self) -> &BindingSignature {
        self.authorization.binding_sig()
    }

    /// Returns the merged tachystamp.
    pub fn tachystamp(&self) -> &Tachystamp {
        self.authorization.tachystamp()
    }
}

// -----------------------------------------------------------------------------
// Common methods
// -----------------------------------------------------------------------------

impl<A: Authorization, V> Bundle<A, V> {
    /// Returns the actions in this bundle.
    pub fn actions(&self) -> &[Action<A::SpendAuth>] {
        &self.actions
    }

    /// Returns the value balance of this bundle.
    pub fn value_balance(&self) -> &V {
        &self.value_balance
    }

    /// Returns the authorization data.
    pub fn authorization(&self) -> &A {
        &self.authorization
    }

    /// Returns the number of actions in this bundle.
    pub fn actions_count(&self) -> usize {
        self.actions.len()
    }
}
