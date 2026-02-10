//! # tachyon
//!
//! The Tachyon shielded transaction protocol.
//!
//! Tachyon is a scaling solution for Zcash that enables:
//! - **Proof Aggregation**: Multiple Halo proofs aggregated into a single Ragu proof per block
//! - **Oblivious Synchronization**: Wallets can outsource sync to untrusted services
//! - **Polynomial Accumulators**: Unified tracking of commitments and nullifiers via tachygrams
//!
//! ## Aggregation States
//!
//! Bundles use a type-state pattern to track tachystamp disposition:
//!
//! - [`Autonome`] - Self-contained bundle with tachystamp (can stand alone)
//! - [`Adjunct`] - Dependent bundle, no tachystamp (depends on aggregate)
//! - [`Aggregate`] - Merged tachystamp covering adjunct bundles (may have own actions)
//!
//! ## Block Structure
//!
//! A block can contain a mix of:
//! - [`Autonome`] - Standalone transactions with their own proof
//! - [`Adjunct`] - Dependent transactions (proof in aggregate)
//! - [`Aggregate`] - Aggregate transaction(s) covering adjunct bundles
//!
//! Multiple aggregates can exist in one block, each covering different bundles.
//! Aggregates may also have their own actions (e.g., miner fee outputs).
//!
//! ## Key Hierarchy
//!
//! Tachyon simplifies the key hierarchy compared to Orchard by removing
//! key diversification, viewing keys, and payment addresses from the core
//! protocol. These capabilities are handled by higher-level wallet software
//! through out-of-band payment protocols.
//!
//! ```mermaid
//! flowchart TB
//!     sk[SpendingKey sk]
//!     ask[ask SigningKey SpendAuth]
//!     nk[NullifierKey nk]
//!     pk[PaymentKey pk]
//!     ak[ak VerificationKey SpendAuth]
//!     pak[ProofAuthorizingKey]
//!     sk --> ask
//!     sk --> nk
//!     sk --> pk
//!     ask --> ak
//!     ak --> pak
//!     nk --> pak
//! ```
//!
//! - **ask**: Authorizes spends (RedPallas signing key)
//! - **ak + nk** (proof authorizing key): Constructs proofs without spend
//!   authority; can be delegated to an oblivious syncing service
//! - **nk**: Observes when funds are spent (nullifier derivation)
//! - **pk**: Used in note construction and out-of-band payment protocols
//!
//! ## Nullifier Derivation
//!
//! Nullifiers are derived via a GGM tree PRF instantiated from Poseidon:
//!
//! $$\mathsf{nf} = F_{\mathsf{nk}}(\Psi \parallel \tau)$$
//!
//! where $\Psi$ is the nullifier trapdoor and $\tau$ is the epoch.
//!
//! The master root key $\mathsf{mk} = \text{KDF}(\Psi, \mathsf{nk})$ supports
//! oblivious sync delegation: prefix keys $\Psi_t$ permit evaluating the PRF
//! only for epochs $e \leq t$, enabling range-restricted delegation without
//! revealing spend capability.
//!
//! ## Nomenclature
//!
//! All types in the `tachyon` crate, unless otherwise specified, are Tachyon-specific
//! types.

#![cfg_attr(docsrs, feature(doc_cfg))]
// Temporary until we have more of the crate implemented.
#![allow(dead_code)]
// Catch documentation errors caused by code changes.
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod action;
pub mod bundle;
pub mod primitives;
pub mod proof;
pub mod stamp;
pub mod value;

pub use action::{Action, RandomizedVerificationKey, SpendAuthSignature};
pub use bundle::{Adjunct, Aggregate, Autonome, BindingSignature, Bundle};
pub use primitives::{NullifierKey, PaymentKey, SpendingKey, Tachygram};
pub use proof::Proof;
pub use stamp::Stamp;
pub use value::ValueCommitment;
