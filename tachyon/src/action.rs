//! Tachyon Action descriptions and authorization traits.

use std::fmt;

use crate::primitives::redpallas;
use crate::value::ValueCommitment;

/// A rerandomized spend authorization verification key (RedPallas).
pub type RandomizedVerificationKey = redpallas::VerificationKey<redpallas::SpendAuth>;

/// A spend authorization signature (RedPallas).
pub type SpendAuthSignature = redpallas::Signature<redpallas::SpendAuth>;

/// A Tachyon Action description.
///
/// An Action transfers value within the Tachyon shielded pool.
/// Unlike Orchard actions which each have their own proof, Tachyon actions
/// are aggregated into a single Ragu proof per block.
///
/// ## Type Parameter
///
/// The type parameter `A` represents the authorization state of this action:
/// - `()` - Unsigned action (no signature yet)
/// - [`SpendAuthSignature`] - Fully authorized action
///
/// ## Fields
///
/// - `cv`: Value commitment to net value (input - output)
/// - `rk`: Randomized spend authorization key
/// - `authorization`: Authorization data (signature or placeholder)
///
/// ## Note
///
/// The tachygram (nullifier or note commitment) is NOT part of the action.
/// Tachygrams are collected separately in the
/// [`Tachystamp`](crate::tachystamp::Tachystamp).  However, `rk` is not a
/// direct input to the Ragu proof -- each `rk` is cryptographically bound to
/// its corresponding tachygram, which *is* a proof input, so the proof
/// validates `rk` transitively.
///
/// This separation allows the tachystamp to be stripped during aggregation
/// while the action (with its signature) remains in the transaction.
#[derive(Clone, Debug)]
pub struct Action<A> {
    /// Value commitment to net value (input - output).
    pub(crate) cv: ValueCommitment,

    /// Randomized spend authorization key.
    pub(crate) rk: RandomizedVerificationKey,

    /// Authorization data for this action.
    ///
    /// This is the spend authorization signature once the action is signed,
    /// or a placeholder type during construction.
    pub(crate) authorization: A,
}

impl<A> Action<A> {
    /// Maps the authorization data using the provided function.
    pub fn map<B>(self, f: impl FnOnce(A) -> B) -> Action<B> {
        Action {
            cv: self.cv,
            rk: self.rk,
            authorization: f(self.authorization),
        }
    }

    /// Tries to map the authorization data, returning an error if the function fails.
    pub fn try_map<B, E>(self, f: impl FnOnce(A) -> Result<B, E>) -> Result<Action<B>, E> {
        Ok(Action {
            cv: self.cv,
            rk: self.rk,
            authorization: f(self.authorization)?,
        })
    }
}

/// Marker type for unsigned actions.
///
/// Used as the authorization type parameter when actions are being constructed
/// but not yet signed.
#[derive(Clone, Copy, Debug, Default)]
pub struct Unsigned;

impl fmt::Display for Unsigned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unsigned")
    }
}
