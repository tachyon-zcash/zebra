//! Tachyon-related functionality.
//!
//! Tachyon is a scaling solution for Zcash that introduces:
//! - Tachygrams: Unified 32-byte blobs (nullifiers or note commitments)
//! - Actions: Spend/output operations with cv, rk, and signature
//! - Stamps: Proof + tachygrams + anchor
//! - Aggregate proof transactions via Ragu PCD
//! - Out-of-band payment distribution (no ciphertexts on-chain)
//!
//! ## Type Structure
//!
//! ```text
//! ShieldedData
//! ├── value_balance: Amount
//! ├── actions: Vec<Action>
//! │   └── Action
//! │       ├── cv: ValueCommitment
//! │       ├── rk: RandomizedVerificationKey
//! │       └── sig: SpendAuthSignature
//! ├── binding_sig: Option<BindingSignature>
//! └── stamp: Option<Stamp>
//!     └── Stamp
//!         ├── tachygrams: Vec<Tachygram>
//!         ├── proof: Proof
//!         └── anchor: Anchor
//! ```
//!
//! ## Crate Organization
//!
//! Protocol types are re-exported from the `tachyon` crate. The only
//! zebra-specific type is [`ShieldedData`] (bundle with runtime stamp
//! optionality and serde).

#![warn(missing_docs)]

#[cfg(any(test, feature = "proptest-impl"))]
mod arbitrary;

pub mod shielded_data;

// Re-export protocol types from tachyon crate
pub use zcash_tachyon::{
    Action, Anchor, BindingSignature, BindingVerificationKey, Epoch, Proof,
    RandomizedVerificationKey, SpendAuthSignature, Stamp, Tachygram,
};
pub use zcash_tachyon::value::Commitment as ValueCommitment;

// Zebra-specific types
pub use shielded_data::ShieldedData;