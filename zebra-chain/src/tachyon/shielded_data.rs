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
use byteorder::{LittleEndian, WriteBytesExt};

use crate::{amount::{Amount, NegativeAllowed}, serialization::ZcashSerialize};

/// Tachyon shielded data bundle for a transaction.
///
/// Uses tachyon crate types directly. Serde is implemented at the bundle
/// level — individual tachyon types do not carry serde derives.
#[derive(Clone)]
pub struct ShieldedData {
    /// The actions (cv, rk, sig for each).
    pub actions: Vec<super::Action>,

    /// Net value of Tachyon spends minus outputs.
    pub value_balance: Amount<NegativeAllowed>,

    /// Binding signature on transaction sighash.
    /// None when actions is empty (nothing to sign over).
    pub binding_sig: Option<super::BindingSignature>,

    /// The stamp containing tachygrams, anchor, and proof.
    /// None after stripping during aggregation.
    pub stamp: Option<super::Stamp>,
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

impl PartialEq for ShieldedData {
    fn eq(&self, other: &Self) -> bool {
        // Compare actions by their serialized representation
        if self.actions.len() != other.actions.len() {
            return false;
        }
        
        for (a1, a2) in self.actions.iter().zip(other.actions.iter()) {
            let cv1: [u8; 32] = a1.cv.into();
            let cv2: [u8; 32] = a2.cv.into();
            let rk1: [u8; 32] = a1.rk.into();
            let rk2: [u8; 32] = a2.rk.into();
            let sig1: [u8; 64] = a1.sig.into();
            let sig2: [u8; 64] = a2.sig.into();
            
            if cv1 != cv2 || rk1 != rk2 || sig1 != sig2 {
                return false;
            }
        }
        
        // Compare value balance
        if self.value_balance != other.value_balance {
            return false;
        }
        
        // Compare binding signature
        match (&self.binding_sig, &other.binding_sig) {
            (None, None) => {},
            (Some(s1), Some(s2)) => {
                let sig1: [u8; 64] = (*s1).into();
                let sig2: [u8; 64] = (*s2).into();
                if sig1 != sig2 {
                    return false;
                }
            },
            _ => return false,
        }
        
        match (&self.stamp, &other.stamp) {
            (None, None) => true,
            (Some(s1), Some(s2)) => {
                let proof1: [u8; 192] = s1.proof.into();
                let proof2: [u8; 192] = s2.proof.into();
                s1.tachygrams == s2.tachygrams
                    && s1.anchor == s2.anchor
                    && proof1 == proof2
            },
            _ => false,
        }
    }
}

impl Eq for ShieldedData {}

impl ShieldedData {
    /// Iterate over the actions in this bundle.
    pub fn actions(&self) -> impl Iterator<Item = &super::Action> {
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
    pub fn binding_verification_key(&self) -> Option<super::BindingVerificationKey> {
        if self.actions.is_empty() {
            return None;
        }

        Some(super::BindingVerificationKey::derive(
            &self.actions,
            self.value_balance.into(),
        ))
    }
}

use serde_big_array::BigArray;

/// Serde proxy for a single Action.
#[derive(Serialize, Deserialize)]
struct ActionProxy {
    cv: [u8; 32],
    rk: [u8; 32],
    #[serde(with = "BigArray")]
    sig: [u8; 64],
}

impl From<super::Action> for ActionProxy {
    fn from(a: super::Action) -> Self {
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
                let cv = super::ValueCommitment::try_from(&a.cv)
                    .map_err(serde::de::Error::custom)?;
                let rk = super::RandomizedVerificationKey::try_from(a.rk)
                    .map_err(serde::de::Error::custom)?;
                let sig = super::SpendAuthSignature::from(a.sig);
                Ok(super::Action { cv, rk, sig })
            })
            .collect::<Result<Vec<_>, D::Error>>()?;

        let binding_sig = proxy
            .binding_sig
            .map(|b| super::BindingSignature::from(b.0));

        let stamp = proxy
            .stamp
            .map(|s| {
                let tachygrams = s
                    .tachygrams
                    .into_iter()
                    .map(|bytes| {
                        <Option<pallas::Base>>::from(pallas::Base::from_repr(bytes))
                            .map(|fp| super::Tachygram::from(fp))
                            .ok_or_else(|| serde::de::Error::custom("invalid field element"))
                    })
                    .collect::<Result<Vec<_>, D::Error>>()?;

                let anchor = <Option<pallas::Base>>::from(pallas::Base::from_repr(s.anchor))
                    .map(|fp| super::Anchor::from(fp))
                    .ok_or_else(|| serde::de::Error::custom("invalid field element"))?;

                let proof =
                    super::Proof::try_from(&s.proof).map_err(serde::de::Error::custom)?;

                Ok(super::Stamp {
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

impl ZcashSerialize for ShieldedData {
    fn zcash_serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
        // For now, implement a basic serialization
        // This is a placeholder implementation - the actual format should follow 
        // the Tachyon specification when it's defined
        
        // Serialize number of actions
        writer.write_u32::<LittleEndian>(self.actions.len() as u32)?;
        
        // Serialize each action (cv, rk, sig)
        for action in &self.actions {
            let cv_bytes: [u8; 32] = action.cv.into();
            let rk_bytes: [u8; 32] = action.rk.into();
            let sig_bytes: [u8; 64] = action.sig.into();
            
            writer.write_all(&cv_bytes)?;
            writer.write_all(&rk_bytes)?;
            writer.write_all(&sig_bytes)?;
        }
        
        // Serialize value balance
        let balance_i64: i64 = self.value_balance.into();
        writer.write_i64::<LittleEndian>(balance_i64)?;
        
        // Serialize binding signature (optional)
        match &self.binding_sig {
            Some(sig) => {
                writer.write_u8(1)?;
                let sig_bytes: [u8; 64] = (*sig).into();
                writer.write_all(&sig_bytes)?;
            }
            None => {
                writer.write_u8(0)?;
            }
        }
        
        // Serialize stamp (optional)
        match &self.stamp {
            Some(_) => {
                writer.write_u8(1)?;
                // TODO: Implement stamp serialization when format is defined
            }
            None => {
                writer.write_u8(0)?;
            }
        }
        
        Ok(())
    }
}