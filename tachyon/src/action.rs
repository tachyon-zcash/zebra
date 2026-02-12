//! Tachyon Action descriptions.

use std::iter::Sum;
use std::ops;
use std::sync::LazyLock;

use ff::Field;
use group::GroupEncoding;
use pasta_curves::{arithmetic::CurveExt, pallas};
use rand::{CryptoRng, RngCore};

use crate::keys::{
    Binding, Signature, SigningKey, SpendAuth, VALUE_COMMITMENT_DOMAIN, VerificationKey,
};
use crate::note::{Note, Nullifier};
use crate::primitives::{Epoch, Fq, PallasPoint};
use crate::proof::ActionWitness;

/// A rerandomized spend authorization verification key (RedPallas).
pub type RandomizedVerificationKey = VerificationKey<SpendAuth>;

/// A spend authorization signature (RedPallas).
pub type SpendAuthSignature = Signature<SpendAuth>;

/// A Tachyon Action description.
///
///
/// ## Fields
///
/// - `cv`: Value commitment to net value (input - output)
/// - `rk`: Randomized spend authorization key
/// - `sig`: RedPallas authorization by `rk`
///
/// ## Note
///
/// The tachygram (nullifier or note commitment) is NOT part of the action.
/// Tachygrams are collected separately in the
/// [`Stamp`](crate::stamp::Stamp).  However, `rk` is not a
/// direct input to the Ragu proof -- each `rk` is cryptographically bound to
/// its corresponding tachygram, which *is* a proof input, so the proof
/// validates `rk` transitively.
///
/// This separation allows the stamp to be stripped during aggregation
/// while the action (with its authorization) remains in the transaction.
#[derive(Clone, Debug)]
pub struct Action {
    /// Value commitment to net value (input - output).
    pub cv: ValueCommitment,

    /// Randomized spend authorization key.
    pub rk: RandomizedVerificationKey,

    /// RedPallas authorization by `rk`.
    pub sig: SpendAuthSignature,
}

impl Action {
    /// Consume a note.
    pub fn spend<R: RngCore + CryptoRng>(
        ask: &SigningKey<SpendAuth>,
        note: Note,
        nf: Nullifier,
        flavor: Epoch,
        rng: &mut R,
    ) -> (Self, ActionWitness) {
        let v = note.value as i64;
        let token = nf.0;

        let alpha = Fq::random(&mut *rng);
        let rsk = ask.randomize(&alpha);
        let rcv = ValueCommitTrapdoor(Fq::random(&mut *rng));
        let rk: RandomizedVerificationKey = (&rsk).into();
        let cv = ValueCommitment::commit(v, rcv.0);

        // Sign cv || rk
        let cv_bytes: [u8; 32] = cv.0.to_bytes();
        let rk_bytes: [u8; 32] = (&rk).into();
        let mut msg = [0u8; 64];
        msg[..32].copy_from_slice(&cv_bytes);
        msg[32..].copy_from_slice(&rk_bytes);
        let sig = rsk.sign(&mut *rng, &msg);

        (
            Action { cv, rk, sig },
            ActionWitness {
                note,
                value: v,
                alpha,
                rcv,
                flavor,
                token,
            },
        )
    }

    /// Create a note.
    pub fn output<R: RngCore + CryptoRng>(
        note: Note,
        flavor: Epoch,
        rng: &mut R,
    ) -> (Self, ActionWitness) {
        let value = -(note.value as i64);
        let token = note.commitment().0;

        let alpha = Fq::random(&mut *rng);
        let rsk: SigningKey<SpendAuth> = {
            use ff::PrimeField;
            alpha
                .to_repr()
                .try_into()
                .expect("random scalar yields valid signing key")
        };
        let rcv = ValueCommitTrapdoor(Fq::random(&mut *rng));
        let rk: RandomizedVerificationKey = (&rsk).into();
        let cv = ValueCommitment::commit(value, rcv.0);

        // Sign cv || rk
        let cv_bytes: [u8; 32] = cv.0.to_bytes();
        let rk_bytes: [u8; 32] = (&rk).into();
        let mut msg = [0u8; 64];
        msg[..32].copy_from_slice(&cv_bytes);
        msg[32..].copy_from_slice(&rk_bytes);
        let sig = rsk.sign(&mut *rng, &msg);

        (
            Action { cv, rk, sig },
            ActionWitness {
                note,
                value,
                alpha,
                rcv,
                flavor,
                token,
            },
        )
    }
}

/// Generator V for value commitments.
#[allow(non_snake_case)]
static VALUE_COMMIT_V: LazyLock<pallas::Point> =
    LazyLock::new(|| pallas::Point::hash_to_curve(VALUE_COMMITMENT_DOMAIN)(b"v"));

/// Generator R for value commitments and binding signatures.
#[allow(non_snake_case)]
static VALUE_COMMIT_R: LazyLock<pallas::Point> =
    LazyLock::new(|| pallas::Point::hash_to_curve(VALUE_COMMITMENT_DOMAIN)(b"r"));

/// A value commitment for a Tachyon action.
///
/// Commits to the value being transferred in an action without revealing it.
/// This is a Pedersen commitment (curve point) used in value balance verification.
///
/// The commitment has the form: `[v] V + [rcv] R` where:
/// - `v` is the value
/// - `rcv` is the randomness
/// - `V` and `R` are generator points derived from [`VALUE_COMMITMENT_DOMAIN`]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValueCommitment(pub PallasPoint);

impl ValueCommitment {
    /// Create a value commitment from a signed value and randomness.
    ///
    /// `cv = [v] V + [rcv] R`
    ///
    /// Positive for spends (balance contributed), negative for outputs (balance exhausted).
    #[allow(non_snake_case)]
    pub fn commit(v: i64, rcv: Fq) -> Self {
        let V = *VALUE_COMMIT_V;
        let R = *VALUE_COMMIT_R;
        let scalar = if v >= 0 {
            Fq::from(v as u64)
        } else {
            -Fq::from((-v) as u64)
        };
        Self((V * scalar + R * rcv).into())
    }

    /// Create the value balance commitment `[value_balance] V`.
    ///
    /// This is `commit(value_balance, 0)` — a deterministic commitment with
    /// no randomness. Used by validators to derive the binding verification key:
    ///
    /// `bvk = sum(cv_i) - ValueCommitment::balance(value_balance)`
    pub fn balance(value_balance: i64) -> Self {
        Self::commit(value_balance, Fq::ZERO)
    }

    /// Convert this value commitment point into a binding verification key.
    ///
    /// Follows Orchard's `into_bvk()` pattern. Typically called on the result
    /// of `sum(cv_i) - ValueCommitment::balance(value_balance)`.
    pub fn into_bvk(self) -> VerificationKey<Binding> {
        let bytes: [u8; 32] = self.into();
        bytes
            .try_into()
            .expect("valid curve point yields valid verification key")
    }
}

impl From<ValueCommitment> for [u8; 32] {
    fn from(cv: ValueCommitment) -> Self {
        cv.0.to_bytes()
    }
}

impl TryFrom<[u8; 32]> for ValueCommitment {
    type Error = &'static str;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        let point = PallasPoint::from_bytes(&bytes);
        if point.is_some().into() {
            Ok(Self(point.unwrap()))
        } else {
            Err("Invalid pallas::Affine for ValueCommitment")
        }
    }
}

impl ops::Add for ValueCommitment {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self((self.0 + rhs.0).into())
    }
}

impl ops::Sub for ValueCommitment {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self((self.0 - rhs.0).into())
    }
}

impl Sum for ValueCommitment {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        use group::prime::PrimeCurveAffine;
        iter.fold(ValueCommitment(PallasPoint::identity()), ops::Add::add)
    }
}

/// The blinding factor (trapdoor) for a value commitment.
///
/// Each action's `rcv` must be accumulated to compute the binding signing key:
/// `bsk = sum(rcv_i)`. The binding signature proves the signer knew all
/// trapdoors, which transitively proves value balance.
///
/// See [`ValueCommitment::commit`] for how `rcv` enters the commitment.
#[derive(Clone, Copy, Debug)]
pub struct ValueCommitTrapdoor(pub Fq);

impl ops::Add for ValueCommitTrapdoor {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sum for ValueCommitTrapdoor {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(ValueCommitTrapdoor(Fq::ZERO), ops::Add::add)
    }
}

impl ValueCommitTrapdoor {
    /// Convert the accumulated trapdoor into a binding signing key.
    ///
    /// Follows Orchard's `into_bsk()` pattern: the caller sums all `rcv`
    /// values from actions, then calls this to obtain a signing key that
    /// uses the same basepoint R as the value commitment scheme.
    pub fn into_bsk(self) -> SigningKey<Binding> {
        use ff::PrimeField;
        let bytes: [u8; 32] = self.0.to_repr();
        bytes
            .try_into()
            .expect("nonzero trapdoor sum yields valid signing key")
    }
}
