//! Tachyon transaction bundles.
//!
//! A bundle is parameterized by its stamp state:
//!
//! - [`StampedBundle`] (`Bundle<Stamp>`) — has a stamp, can stand alone or cover adjuncts
//! - [`StrippedBundle`] (`Bundle<Adjunct>`) — stamp stripped, depends on a stamped bundle
//!
//! Actions are constant through state transitions; only the stamp
//! is stripped or merged.
//!
//! ## Aggregation
//!
//! An aggregate is a `(StampedBundle, Vec<StrippedBundle>)` — the stamped
//! bundle's stamp covers both its own actions and those of the stripped bundles.

use crate::action::Action;
use crate::keys::{Binding, Signature};
use crate::stamp::Stamp;
use crate::value::ValueCommitment;

/// A binding signature for value balance verification (RedPallas).
pub type BindingSignature = Signature<Binding>;

/// Marker for the absence of a stamp.
///
/// A `Bundle<Adjunct>` has had its stamp stripped and depends on a
/// [`StampedBundle`] in the same block.
#[derive(Clone)]
pub struct Adjunct;

/// A Tachyon transaction bundle parameterized by stamp state `S`.
///
/// - `Bundle<Stamp>` ([`StampedBundle`]) — self-contained with stamp
/// - `Bundle<Adjunct>` ([`StrippedBundle`]) — stamp stripped, dependent
#[derive(Clone)]
pub struct Bundle<S> {
    /// Tachyactions (cv, rk, sig).
    actions: Vec<Action>,

    /// Net value of spends minus outputs.
    value_balance: ValueCommitment,

    /// Binding signature on transaction sighash.
    binding_sig: BindingSignature,

    /// Stamp state: `Stamp` when present, `Adjunct` when stripped.
    stamp: S,
}

/// A bundle with a stamp — can stand alone or cover adjunct bundles.
pub type StampedBundle = Bundle<Stamp>;

/// A bundle whose stamp has been stripped — depends on a stamped bundle.
pub type StrippedBundle = Bundle<Adjunct>;

// =============================================================================
// Common methods (all bundle states)
// =============================================================================

impl<S> Bundle<S> {
    /// Returns the actions in this bundle.
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// Returns the value balance.
    pub fn value_balance(&self) -> &ValueCommitment {
        &self.value_balance
    }

    /// Returns the binding signature.
    pub fn binding_sig(&self) -> &BindingSignature {
        &self.binding_sig
    }
}

// =============================================================================
// StampedBundle methods
// =============================================================================

impl StampedBundle {
    /// Creates a new stamped bundle.
    pub fn new(
        actions: Vec<Action>,
        value_balance: ValueCommitment,
        binding_sig: BindingSignature,
        stamp: Stamp,
    ) -> Self {
        Bundle {
            actions,
            value_balance,
            binding_sig,
            stamp,
        }
    }

    /// Returns the stamp.
    pub fn stamp(&self) -> &Stamp {
        &self.stamp
    }

    /// Strips the stamp, producing a stripped bundle and the extracted stamp.
    ///
    /// The stamp should be merged into an aggregate's stamped bundle.
    pub fn strip(self) -> (StrippedBundle, Stamp) {
        (
            Bundle {
                actions: self.actions,
                value_balance: self.value_balance,
                binding_sig: self.binding_sig,
                stamp: Adjunct,
            },
            self.stamp,
        )
    }
}

// =============================================================================
// StrippedBundle methods
// =============================================================================

impl StrippedBundle {
    /// Creates a new stripped bundle.
    pub fn new(
        actions: Vec<Action>,
        value_balance: ValueCommitment,
        binding_sig: BindingSignature,
    ) -> Self {
        Bundle {
            actions,
            value_balance,
            binding_sig,
            stamp: Adjunct,
        }
    }
}
