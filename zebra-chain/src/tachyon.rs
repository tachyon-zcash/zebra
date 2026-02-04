//! Tachyon-related functionality.
//!
//! Tachyon is a scaling solution for Zcash that introduces:
//! - Tachygrams: Unified 32-byte blobs (nullifiers or note commitments)
//! - Tachyactions: Spend/output operations with cv, rk, and signature
//! - Tachystamps: Proof + tachygrams + anchor
//! - Aggregate proof transactions via Ragu PCD
//! - Out-of-band payment distribution (no ciphertexts on-chain)
//!
//! ## Authorization States
//!
//! Bundles use a type-state pattern to track progress:
//!
//! - [`Unsigned`] - Bundle being constructed, no signatures yet
//! - [`Autonome`] - Self-contained bundle with tachystamp (can stand alone)
//! - [`Adjunct`] - Dependent bundle, no tachystamp (depends on aggregate)
//! - [`Aggregate`] - Merged tachystamp covering adjunct bundles (may have own actions)
//!
//! ## Block Structure
//!
//! A block can contain a mix of:
//! - `Bundle<Autonome, V>` - Standalone transactions with their own proof
//! - `Bundle<Adjunct, V>` - Dependent transactions (proof in aggregate)
//! - `Bundle<Aggregate, V>` - Aggregate transaction(s) covering adjunct bundles
//!
//! Multiple aggregates can exist in one block, each covering different bundles.
//! Aggregates may also have their own actions (e.g., miner fee outputs).
//!
//! ## Type Structure
//!
//! ```text
//! ShieldedData (Tachyon Bundle)
//! ├── value_balance: Amount
//! ├── actions: AtLeastOne<Tachyaction>
//! │   └── Tachyaction
//! │       ├── cv: ValueCommitment
//! │       ├── rk: VerificationKeyBytes
//! │       └── spend_auth_sig: Signature
//! ├── binding_sig: Signature
//! └── tachystamp: Option<Tachystamp>
//!     └── Tachystamp
//!         ├── tachygrams: Vec<Tachygram>
//!         ├── proof: AggregateProof
//!         └── anchor: Anchor
//! ```
//!
//! ## Crate Organization
//!
//! This module provides two categories of types:
//!
//! ### Protocol Types (re-exported from `tachyon` crate)
//!
//! Core protocol types used for transaction construction:
//!
//! - [`Epoch`] - Epoch identifier for nullifier flavoring
//! - [`Nullifier`] - Nullifier value (without epoch)
//! - [`Anchor`] - A recent state identifier
//! - [`Authorization`] - Trait for bundle authorization states
//! - [`Unsigned`], [`Autonome`], [`Adjunct`], [`Aggregate`] - Authorization state types
//!
//! ### Blockchain Types (defined here)
//!
//! Types with ZcashSerialize/ZcashDeserialize for blockchain storage:
//!
//! - [`ShieldedData`] - Tachyon bundle (value_balance, actions, binding_sig, optional tachystamp)
//! - [`Tachystamp`] - Proof + tachygrams + anchor
//! - [`Tachyaction`] - cv + rk + spend_auth_sig
//! - [`ValueCommitment`] - Homomorphic commitment with Add/Sub/Sum
//! - [`Tachygram`] - 32-byte blob for accumulator entries
//! - [`Anchor`](accumulator::Anchor) - Serializable accumulator root

#![warn(missing_docs)]

mod action;
mod commitment;
mod nullifier;
mod proof;
mod tachygram;

#[cfg(any(test, feature = "proptest-impl"))]
mod arbitrary;

pub mod accumulator;
pub mod shielded_data;

// Re-export protocol types from tachyon crate
pub use tachyon::{Adjunct, Aggregate, Authorization, Autonome, Epoch, Nullifier, Unsigned};

// Blockchain-specific types with ZcashSerialize/Deserialize
pub use action::Tachyaction;
pub use commitment::{NoteCommitment, ValueCommitment};
pub use nullifier::FlavoredNullifier;
pub use proof::AggregateProof;
pub use shielded_data::{ShieldedData, Tachystamp};
pub use tachygram::Tachygram;
