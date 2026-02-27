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

use byteorder::{LittleEndian, WriteBytesExt};
use serde::{Deserialize, Serialize};

use crate::{
    amount::{Amount, NegativeAllowed}, 
    serialization::ZcashSerialize
};

/// Tachyon shielded data bundle for a transaction.
///
/// Uses tachyon crate types directly with custom serde implementation.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
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


// =============================================================================
// ZcashSerialize implementation
// =============================================================================

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