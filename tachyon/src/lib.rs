//! # tachyon
//!
//! The Tachyon shielded transaction protocol.
//!
//! Tachyon is a scaling solution for Zcash that enables:
//! - **Proof Aggregation**: Multiple Halo proofs aggregated into a single Ragu proof per block
//! - **Oblivious Synchronization**: Wallets can outsource sync to untrusted services
//! - **Polynomial Accumulators**: Unified tracking of commitments and nullifiers via tachygrams
//!
//! ## Type Structure
//!
//! ```text
//! Bundle<A, V> (Tachyon Bundle)
//! ├── value_balance: V
//! ├── actions: Vec<Action<A::SpendAuth>>
//! │   └── Action<SpendAuth>
//! │       ├── cv: ValueCommitment
//! │       ├── rk: RandomizedVerificationKey
//! │       └── authorization: SpendAuth
//! └── authorization: A
//!     └── Autonome | Adjunct | Aggregate
//! ```
//!
//! ## Authorization States
//!
//! Bundles use a type-state pattern to track authorization progress:
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
//! ## Nomenclature
//!
//! All types in the `tachyon` crate, unless otherwise specified, are Tachyon-specific
//! types. For example, [`Address`] is a Tachyon payment address, and [`Tachygram`]
//! is a unified commitment/nullifier representation unique to Tachyon.

#![cfg_attr(docsrs, feature(doc_cfg))]
// Temporary until we have more of the crate implemented.
#![allow(dead_code)]
// Catch documentation errors caused by code changes.
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

mod action;
mod address;
pub mod bundle;
pub mod keys;
pub mod note;
pub mod primitives;
pub mod tachygram;
pub mod value;

pub use action::{Action, RandomizedVerificationKey, SpendAuthSignature, Unsigned};
pub use address::Address;
pub use bundle::{
    Adjunct, Aggregate, Authorization, Autonome, BindingSignature, Bundle, Proof, Tachystamp,
};
pub use keys::{FullViewingKey, IncomingViewingKey, NullifierKey, SpendingKey};
pub use note::{Epoch, Note, NoteCommitment, Nullifier, NullifierTrapdoor};
pub use tachygram::Tachygram;
pub use value::ValueCommitment;
