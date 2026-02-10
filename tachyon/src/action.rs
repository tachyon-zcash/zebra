//! Tachyon Action descriptions.

use reddsa::Signature;

use crate::keys::{SpendAuth, VerificationKey};
use crate::primitives::{Fp, NoteValue, PallasPoint, Tachygram};
use crate::value::ValueCommitment;

/// A rerandomized spend authorization verification key (RedPallas).
pub type RandomizedVerificationKey = VerificationKey<SpendAuth>;

/// A spend authorization signature (RedPallas).
pub type SpendAuthSignature = Signature<SpendAuth>;

/// An action paired with its tachygram.
///
/// Each action relates to exactly one tachygram. The pair is the input
/// to proof verification.
pub type Tachyaction = (Action, Tachygram);

/// A Tachyon Action description.
///
/// An Action transfers value within the Tachyon shielded pool.
/// Unlike Orchard actions which each have their own proof, Tachyon actions
/// are aggregated into a single Ragu proof per block.
///
/// ## Fields
///
/// - `cv`: Value commitment to net value (input - output)
/// - `rk`: Randomized spend authorization key
/// - `authorization`: Spend authorization signature
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
#[derive(Clone)]
pub struct Action {
    /// Value commitment to net value (input - output).
    pub(crate) cv: ValueCommitment,

    /// Randomized spend authorization key.
    pub(crate) rk: RandomizedVerificationKey,

    /// Spend authorization signature.
    pub(crate) sig: SpendAuthSignature,
}

impl Action {
    /// Creates a new spend action.
    ///
    /// # Arguments
    ///
    /// * `pk`: The public key of the spend authorization key.
    /// * `v`: The value of the note.
    /// * `psi`: The nullifier trapdoor.
    /// * `rcm`: The note commitment.
    pub fn spend(pk: [u8; 32], _v: NoteValue, _psi: Fp, _rcm: PallasPoint) -> Tachyaction {
        (
            Self {
                cv: ValueCommitment(PallasPoint::default()),
                rk: RandomizedVerificationKey::try_from(pk).unwrap(),
                sig: SpendAuthSignature::from([0; 64]),
            },
            Tachygram(Fp::default()),
        )
    }

    /// Creates a new output action.
    ///
    /// # Arguments
    ///
    /// * `pk`: The public key of the spend authorization key.
    /// * `v`: The value of the note.
    /// * `psi`: The nullifier trapdoor.
    /// * `rcm`: The note commitment.
    pub fn output(pk: [u8; 32], _v: NoteValue, _psi: Fp, _rcm: PallasPoint) -> Tachyaction {
        (
            Self {
                cv: ValueCommitment(PallasPoint::default()),
                rk: RandomizedVerificationKey::try_from(pk).unwrap(),
                sig: SpendAuthSignature::from([0; 64]),
            },
            Tachygram(Fp::default()),
        )
    }
}
