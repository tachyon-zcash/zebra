//! Tachyon transaction bundles and aggregation.
//!
//! Three bundle types model tachystamp disposition, all parameterized
//! by `N` (the number of tachygrams):
//!
//! - [`Autonome<N>`] - N actions, N tachygrams, self-contained
//! - [`Adjunct<N>`] - N actions, stamp stripped
//! - [`Aggregate<N>`] - <N actions, N tachygrams, merged stamp
//!
//! Actions are constant through state transitions; only the stamp
//! (tachygrams, proof, epoch) is stripped or merged.
//!
//! ## Block Structure
//!
//! A block can contain a mix of autonome, adjunct, and aggregate bundles.
//! Multiple aggregates can exist in one block, each covering a different set of
//! adjunct bundles.

use crate::action::Action;
use crate::primitives::{Binding, Signature};
use crate::stamp::Stamp;
use crate::value::ValueCommitment;

/// A binding signature for value balance verification (RedPallas).
pub type BindingSignature = Signature<Binding>;

// =============================================================================
// Bundle trait
// =============================================================================

/// Shared behavior across all bundle aggregation states.
///
/// ## Implementors
///
/// - [`Autonome`] - Self-contained bundle with stamp
/// - [`Adjunct`] - Dependent bundle, stamp stripped
/// - [`Aggregate`] - Merged stamp covering adjunct bundles
pub trait Bundle {
    /// Returns the actions in this bundle.
    fn actions(&self) -> &[Action];

    /// Returns the value balance.
    fn value_balance(&self) -> &ValueCommitment;

    /// Returns the binding signature.
    fn binding_sig(&self) -> &BindingSignature;
}

// =============================================================================
// Autonome
// =============================================================================

/// Self-contained bundle: N actions with N tachygrams.
///
/// An autonome bundle has everything needed to validate independently:
/// - N actions with spend authorization signatures
/// - Binding signature
/// - Stamp with N tachygrams, N-input proof, and anchor
///
/// Autonome bundles can appear directly in blocks without an aggregate.
#[derive(Clone)]
pub struct Autonome<const N: usize> {
    /// N tachyactions (cv, rk, sig).
    actions: [Action; N],

    /// Net value of spends minus outputs.
    value_balance: ValueCommitment,

    /// Binding signature on transaction sighash.
    binding_sig: BindingSignature,

    /// The stamp (N tachygrams, N-input proof, anchor).
    stamp: Stamp<N>,
}

impl<const N: usize> Autonome<N> {
    /// Creates a new autonome bundle.
    pub fn new(
        actions: [Action; N],
        value_balance: ValueCommitment,
        binding_sig: BindingSignature,
        stamp: Stamp<N>,
    ) -> Self {
        Self {
            actions,
            value_balance,
            binding_sig,
            stamp,
        }
    }

    /// Returns the stamp.
    pub fn stamp(&self) -> &Stamp<N> {
        &self.stamp
    }

    /// Strips the stamp, producing an adjunct and the extracted stamp.
    ///
    /// The stamp should be merged into an [`Aggregate`].
    pub fn strip(self) -> (Adjunct<N>, Stamp<N>) {
        let adjunct = Adjunct {
            actions: self.actions,
            value_balance: self.value_balance,
            binding_sig: self.binding_sig,
        };
        (adjunct, self.stamp)
    }
}

impl<const N: usize> Bundle for Autonome<N> {
    fn actions(&self) -> &[Action] {
        &self.actions
    }

    fn value_balance(&self) -> &ValueCommitment {
        &self.value_balance
    }

    fn binding_sig(&self) -> &BindingSignature {
        &self.binding_sig
    }
}

// =============================================================================
// Adjunct
// =============================================================================

/// Dependent bundle: N actions, stamp stripped.
///
/// An adjunct bundle retains its N actions and binding signature but has
/// no stamp — it was contributed to an [`Aggregate`].
///
/// Adjunct bundles require a corresponding aggregate in the same block.
#[derive(Clone)]
pub struct Adjunct<const N: usize> {
    /// N tachyactions (cv, rk, sig).
    actions: [Action; N],

    /// Net value of spends minus outputs.
    value_balance: ValueCommitment,

    /// Binding signature on transaction sighash.
    binding_sig: BindingSignature,
}

impl<const N: usize> Adjunct<N> {
    /// Creates a new adjunct bundle.
    pub fn new(
        actions: [Action; N],
        value_balance: ValueCommitment,
        binding_sig: BindingSignature,
    ) -> Self {
        Self {
            actions,
            value_balance,
            binding_sig,
        }
    }
}

impl<const N: usize> Bundle for Adjunct<N> {
    fn actions(&self) -> &[Action] {
        &self.actions
    }

    fn value_balance(&self) -> &ValueCommitment {
        &self.value_balance
    }

    fn binding_sig(&self) -> &BindingSignature {
        &self.binding_sig
    }
}

// =============================================================================
// Aggregate
// =============================================================================

/// Merged stamp covering multiple adjunct bundles: N tachygrams, <N actions.
///
/// An aggregate bundle contains:
/// - Merged stamp with N tachygrams, N-input proof, and anchor
/// - Binding signature
/// - Any number of actions below N (0 = innocent, >0 = based)
///
/// Multiple aggregates can exist in a single block, each covering a
/// different set of adjunct bundles.
#[derive(Clone)]
pub struct Aggregate<const N: usize> {
    /// Tachyactions (cv, rk, sig). len() < N. May be empty (innocent aggregate).
    actions: Vec<Action>,

    /// Net value of spends minus outputs.
    value_balance: ValueCommitment,

    /// Binding signature on transaction sighash.
    binding_sig: BindingSignature,

    /// The merged stamp (N tachygrams, N-input proof, anchor).
    stamp: Stamp<N>,
}

impl<const N: usize> Aggregate<N> {
    /// Creates a new aggregate bundle.
    pub fn new(
        actions: Vec<Action>,
        value_balance: ValueCommitment,
        binding_sig: BindingSignature,
        stamp: Stamp<N>,
    ) -> Self {
        Self {
            actions,
            value_balance,
            binding_sig,
            stamp,
        }
    }

    /// Returns the merged stamp.
    pub fn stamp(&self) -> &Stamp<N> {
        &self.stamp
    }
}

impl<const N: usize> Bundle for Aggregate<N> {
    fn actions(&self) -> &[Action] {
        &self.actions
    }

    fn value_balance(&self) -> &ValueCommitment {
        &self.value_balance
    }

    fn binding_sig(&self) -> &BindingSignature {
        &self.binding_sig
    }
}
