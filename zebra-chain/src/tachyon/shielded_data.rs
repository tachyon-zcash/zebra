//! Tachyon shielded data for transactions.
//!
//! [`ShieldedData`] is the zebra-chain representation of a Tachyon bundle,
//! using tachyon crate types directly for its fields. Serde is implemented
//! at the bundle level via a proxy struct pattern.
//!
//! ## Stamp optionality
//!
//! A bundle's stamp is `Option<Stamp>`:
//! - `Some`: bundle carries its own proof and tachygrams
//! - `None`: stamp was stripped and merged into another bundle in the same block

use std::fmt;

use halo2::pasta::{group::ff::PrimeField, pallas};

use crate::amount::{Amount, NegativeAllowed};

/// Tachyon shielded data bundle for a transaction.
///
/// Uses tachyon crate types directly. Serde is implemented at the bundle
/// level — individual tachyon types do not carry serde derives.
#[derive(Clone, Eq, PartialEq)]
pub struct ShieldedData {
    /// The actions (cv, rk, sig for each).
    pub actions: Vec<zcash_tachyon::Action>,

    /// Net value of Tachyon spends minus outputs.
    pub value_balance: Amount<NegativeAllowed>,

    /// Binding signature on transaction sighash.
    /// None when actions is empty (nothing to sign over).
    pub binding_sig: Option<zcash_tachyon::BindingSignature>,

    /// The stamp containing tachygrams, anchor, and proof.
    /// None after stripping during aggregation.
    pub stamp: Option<zcash_tachyon::Stamp>,
}

impl fmt::Debug for ShieldedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("Tachyon ShieldedData");
        debug
            .field("actions", &self.actions.len())
            .field("value_balance", &self.value_balance);

        if let Some(ref stamp) = self.stamp {
            debug.field("stamp", &format!("{} tachygrams", stamp.tachygrams.len()));
        } else {
            debug.field("stamp", &"None (stripped)");
        }

        debug.finish()
    }
}

impl fmt::Display for ShieldedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl ShieldedData {
    /// Iterate over the actions in this bundle.
    pub fn actions(&self) -> impl Iterator<Item = &zcash_tachyon::Action> {
        self.actions.iter()
    }

    /// Get the value balance of this bundle.
    pub fn value_balance(&self) -> Amount<NegativeAllowed> {
        self.value_balance
    }

    /// Count the number of actions in this bundle.
    pub fn actions_count(&self) -> usize {
        self.actions.len()
    }

    /// Calculate the binding verification key.
    ///
    /// `bvk = sum(cv_i) - [value_balance] V`
    ///
    /// Follows Orchard's `binding_validating_key()` pattern. The binding
    /// signature proves that the signer knew all value commitment trapdoors,
    /// which transitively proves value balance integrity.
    /// Returns None if there are no actions.
    pub fn binding_verification_key(&self) -> Option<zcash_tachyon::BindingVerificationKey> {
        if self.actions.is_empty() {
            return None;
        }

        Some(zcash_tachyon::BindingVerificationKey::derive(
            &self.actions,
            self.value_balance.into(),
        ))
    }
}

// =============================================================================
// Serde — implemented at the bundle level via proxy structs
// =============================================================================

use serde_big_array::BigArray;

/// Serde proxy for a single Action.
#[derive(Serialize, Deserialize)]
struct ActionProxy {
    cv: [u8; 32],
    rk: [u8; 32],
    #[serde(with = "BigArray")]
    sig: [u8; 64],
}

impl From<zcash_tachyon::Action> for ActionProxy {
    fn from(a: zcash_tachyon::Action) -> Self {
        ActionProxy {
            cv: a.cv.into(),
            rk: a.rk.into(),
            sig: a.sig.into(),
        }
    }
}

/// Serde proxy for a Stamp.
#[derive(Serialize, Deserialize)]
struct StampProxy {
    tachygrams: Vec<[u8; 32]>,
    anchor: [u8; 32],
    #[serde(with = "BigArray")]
    proof: [u8; 192],
}

/// Serde proxy for ShieldedData.
#[derive(Serialize, Deserialize)]
struct ShieldedDataProxy {
    actions: Vec<ActionProxy>,
    value_balance: Amount<NegativeAllowed>,
    binding_sig: Option<BindingSigBytes>,
    stamp: Option<StampProxy>,
}

/// Wrapper for binding signature bytes with serde support.
#[derive(Serialize, Deserialize)]
struct BindingSigBytes(#[serde(with = "BigArray")] [u8; 64]);

impl serde::Serialize for ShieldedData {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let proxy = ShieldedDataProxy {
            actions: self.actions.iter().map(|a| ActionProxy::from(*a)).collect(),
            value_balance: self.value_balance,
            binding_sig: self
                .binding_sig
                .as_ref()
                .map(|s| BindingSigBytes(<[u8; 64]>::from(*s))),
            stamp: self.stamp.as_ref().map(|s| StampProxy {
                tachygrams: s
                    .tachygrams
                    .iter()
                    .map(|tg| pallas::Base::from(*tg).to_repr())
                    .collect(),
                anchor: pallas::Base::from(s.anchor).to_repr(),
                proof: {
                    let bytes: [u8; 192] = s.proof.into();
                    bytes
                },
            }),
        };
        proxy.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for ShieldedData {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let proxy = ShieldedDataProxy::deserialize(deserializer)?;

        let actions = proxy
            .actions
            .into_iter()
            .map(|a| {
                let cv = zcash_tachyon::value::Commitment::try_from(&a.cv)
                    .map_err(serde::de::Error::custom)?;
                let rk = zcash_tachyon::RandomizedVerificationKey::try_from(a.rk)
                    .map_err(serde::de::Error::custom)?;
                let sig = zcash_tachyon::SpendAuthSignature::from(a.sig);
                Ok(zcash_tachyon::Action { cv, rk, sig })
            })
            .collect::<Result<Vec<_>, D::Error>>()?;

        let binding_sig = proxy
            .binding_sig
            .map(|b| zcash_tachyon::BindingSignature::from(b.0));

        let stamp = proxy
            .stamp
            .map(|s| {
                let tachygrams = s
                    .tachygrams
                    .into_iter()
                    .map(|bytes| {
                        <Option<pallas::Base>>::from(pallas::Base::from_repr(bytes))
                            .map(|fp| zcash_tachyon::Tachygram::from(fp))
                            .ok_or_else(|| serde::de::Error::custom("invalid field element"))
                    })
                    .collect::<Result<Vec<_>, D::Error>>()?;

                let anchor = <Option<pallas::Base>>::from(pallas::Base::from_repr(s.anchor))
                    .map(|fp| zcash_tachyon::Anchor::from(fp))
                    .ok_or_else(|| serde::de::Error::custom("invalid field element"))?;

                let proof =
                    zcash_tachyon::Proof::try_from(&s.proof).map_err(serde::de::Error::custom)?;

                Ok(zcash_tachyon::Stamp {
                    tachygrams,
                    anchor,
                    proof,
                })
            })
            .transpose()?;

        Ok(ShieldedData {
            actions,
            value_balance: proxy.value_balance,
            binding_sig,
            stamp,
        })
    }
}
