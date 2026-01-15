//! Sapling shielded coinbase output generation (ZIP 213).
//!
//! This module provides functionality for creating Sapling shielded outputs
//! for coinbase transactions, as specified in [ZIP 213].
//!
//! # Usage
//!
//! The primary way to create shielded coinbase transactions is via the
//! [`Transaction::new_v5_coinbase`] method with `PoolType::Shielded(Sapling)`.
//! This handles proof generation and binding signature internally.
//!
//! For size estimation (used in transaction selection), use
//! [`Transaction::estimate_v5_coinbase`] which uses [`dummy_sapling_shielded_data`]
//! to create placeholder data of the correct size.
//!
//! # Internals
//!
//! Creating shielded coinbase requires solving a chicken-and-egg problem:
//! - The binding signature requires the transaction sighash
//! - The sighash requires the transaction (including Sapling data)
//!
//! Per ZIP 244, the sighash computation includes the Sapling outputs but NOT
//! the binding signature. This allows the builder to:
//! 1. Create outputs and proofs via [`prove_sapling_coinbase`]
//! 2. Build a draft transaction with placeholder binding_sig
//! 3. Compute sighash on the draft
//! 4. Finalize with the real binding signature
//!
//! [ZIP 213]: https://zips.z.cash/zip-0213

use rand_core::RngCore;

use sapling_crypto::{
    builder::{bundle, BundleType, InProgress, OutputInfo, Proven, Unsigned},
    bundle::Bundle,
    keys::OutgoingViewingKey,
    note_encryption::Zip212Enforcement,
    prover::{OutputProver, SpendProver},
    value::NoteValue,
    Anchor, PaymentAddress,
};

use crate::{
    amount::{Amount, NegativeAllowed, NonNegative},
    primitives::{redjubjub::Signature, Groth16Proof},
    sapling::{
        commitment::ValueCommitment,
        keys::EphemeralPublicKey,
        note::{EncryptedNote, WrappedNoteKey},
        Output, SharedAnchor, ShieldedData, TransferData,
    },
    serialization::AtLeastOne,
};

use rand::rngs::OsRng;

/// ZIP 213 specifies that shielded coinbase outputs must use an all-zeros
/// outgoing viewing key for public auditability of the monetary supply.
pub const ZIP213_COINBASE_OVK: OutgoingViewingKey = OutgoingViewingKey([0u8; 32]);

/// Creates proofs for a Sapling coinbase output.
///
/// Returns:
/// - The proven bundle (used to compute the binding signature)
/// - The proven outputs (to include in the transaction's ShieldedData)
/// - The value balance (negative, as value flows INTO the shielded pool)
///
/// This is an internal function used by `Transaction::new_v5_coinbase` and
/// `Transaction::new_v6_coinbase`. External callers should use those methods
/// instead, which handle the full two-phase signing process.
pub(crate) fn prove<R, SP, OP>(
    rng: &mut R,
    recipient: PaymentAddress,
    amount: Amount<NonNegative>,
    spend_prover: &SP,
    output_prover: &OP,
) -> (
    Bundle<InProgress<Proven, Unsigned>, i64>,
    Vec<Output>,
    Amount<NegativeAllowed>,
)
where
    R: RngCore,
    SP: SpendProver,
    OP: OutputProver,
{
    // Convert amount to NoteValue
    let note_value = NoteValue::from_raw(
        u64::try_from(amount.zatoshis()).expect("amount is valid for Sapling note"),
    );

    // Create output info with ZIP 213 all-zeros OVK
    let output_info = OutputInfo::new(
        Some(ZIP213_COINBASE_OVK),
        recipient,
        note_value,
        [0u8; 512], // Empty memo
    );

    // Create the coinbase bundle
    let (unauthorized_bundle, _metadata) = bundle::<SP, OP, _, i64>(
        &mut *rng,
        BundleType::Coinbase,
        Zip212Enforcement::On,
        Anchor::empty_tree(),
        vec![],
        vec![output_info],
        &[],
    )
    .expect("bundle creation should succeed")
    .expect("bundle is not empty");

    // Create proofs for the outputs
    let bundle = unauthorized_bundle.create_proofs(spend_prover, output_prover, &mut *rng, ());

    // Convert outputs to Zebra format
    let proven_outputs: Vec<Output> = bundle
        .shielded_outputs()
        .iter()
        .map(|desc| Output {
            cv: ValueCommitment(desc.cv().clone()),
            cm_u: *desc.cmu(),
            ephemeral_key: EphemeralPublicKey::try_from(desc.ephemeral_key().0)
                .expect("ephemeral key is valid"),
            enc_ciphertext: EncryptedNote(*desc.enc_ciphertext()),
            out_ciphertext: WrappedNoteKey(*desc.out_ciphertext()),
            zkproof: Groth16Proof(*desc.zkproof()),
        })
        .collect();

    // value_balance is negative for coinbase (value flows INTO shielded pool)
    let proven_balance = (-amount)
        .constrain::<NegativeAllowed>()
        .expect("amount is valid for Sapling note");

    (bundle, proven_outputs, proven_balance)
}

/// Builds Sapling shielded data for a coinbase transaction.
///
/// Returns the shielded data (with placeholder binding signature) and the proven
/// bundle needed to compute the real binding signature later.
pub fn build_shielded_coinbase(
    miner_address: PaymentAddress,
    miner_reward: Amount<NonNegative>,
) -> (
    ShieldedData<SharedAnchor>,
    Bundle<InProgress<Proven, Unsigned>, i64>,
) {
    let prover = zcash_proofs::prover::LocalTxProver::bundled();
    let (bundle, proven_outputs, proven_balance) =
        prove(&mut OsRng, miner_address, miner_reward, &prover, &prover);

    (
        ShieldedData {
            value_balance: proven_balance,
            transfers: TransferData::JustOutputs {
                outputs: AtLeastOne::try_from(proven_outputs).expect("outputs are valid"),
            },
            binding_sig: Signature::from([0u8; 64]),
        },
        bundle,
    )
}

/// Creates dummy Sapling shielded data for size estimation.
///
/// All Sapling fields are fixed-size, so this placeholder serializes to the
/// same size as real shielded data. Used by coinbase template estimation.
pub fn dummy_shielded_coinbase(miner_reward: Amount<NonNegative>) -> ShieldedData<SharedAnchor> {
    ShieldedData {
        value_balance: (-miner_reward)
            .constrain::<NegativeAllowed>()
            .expect("miner_reward is valid"),
        transfers: TransferData::JustOutputs {
            outputs: AtLeastOne::try_from(vec![Output::dummy()]).expect("literal"),
        },
        binding_sig: Signature::from([0u8; 64]),
    }
}
