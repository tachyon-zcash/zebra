//! Methods for building transactions.

use zcash_keys::address::Address;
use zcash_protocol::{PoolType, ShieldedProtocol};
use zcash_script::script::Evaluable;

use crate::{
    amount::{Amount, NonNegative},
    block::Height,
    orchard,
    parameters::{Network, NetworkUpgrade},
    sapling,
    transaction::{LockTime, Transaction},
    transparent,
};

use sapling_crypto::{
    builder::{InProgress, Proven, Unsigned},
    bundle::Bundle,
};

#[cfg(feature = "shielded-mining-sapling")]
use {crate::transaction::HashType, rand::rngs::OsRng};

impl Transaction {
    /// Returns a new version 6 coinbase transaction for `network` and `height`,
    /// with the miner reward going to `miner_address` via `miner_pool`.
    #[cfg(all(zcash_unstable = "nu7", feature = "tx_v6"))]
    pub fn new_v6_coinbase(
        network: &Network,
        height: Height,
        outputs: Vec<(Amount<NonNegative>, transparent::Script)>,
        miner_reward: Amount<NonNegative>,
        miner_pool: PoolType,
        miner_address: &Address,
        miner_data: Vec<u8>,
        zip233_amount: Option<Amount<NonNegative>>,
    ) -> Transaction {
        let (outputs, sapling_shielded_data, sapling_proven_bundle) =
            build_coinbase_outputs(miner_pool, miner_address, miner_reward, outputs);

        let orchard_shielded_data: Option<orchard::ShieldedData> = None;

        // # Consensus
        //
        // These consensus rules apply to v6 coinbase transactions:
        //
        // > If effectiveVersion ≥ 5 then this condition MUST hold:
        // > tx_in_count > 0 or nSpendsSapling > 0 or
        // > (nActionsOrchard > 0 and enableSpendsOrchard = 1).
        //
        // > A coinbase transaction for a block at block height greater than 0 MUST have
        // > a script that, as its first item, encodes the block height as follows. ...
        // > let heightBytes be the signed little-endian representation of height,
        // > using the minimum nonzero number of bytes such that the most significant byte
        // > is < 0x80. The length of heightBytes MUST be in the range {1 .. 5}.
        // > Then the encoding is the length of heightBytes encoded as one byte,
        // > followed by heightBytes itself. This matches the encoding used by Bitcoin
        // > in the implementation of [BIP-34]
        // > (but the description here is to be considered normative).
        //
        // > A coinbase transaction script MUST have length in {2 .. 100} bytes.
        //
        // Zebra adds extra coinbase data if configured to do so.
        //
        // Since we're not using a lock time, any sequence number is valid here.
        // See `Transaction::lock_time()` for the relevant consensus rules.
        //
        // <https://zips.z.cash/protocol/protocol.pdf#txnconsensus>
        let inputs = vec![transparent::Input::new_coinbase(height, miner_data, None)];

        // > The block subsidy is composed of a miner subsidy and a series of funding streams.
        //
        // <https://zips.z.cash/protocol/protocol.pdf#subsidyconcepts>
        //
        // > The total value in zatoshi of transparent outputs from a coinbase transaction,
        // > minus vbalanceSapling, minus vbalanceOrchard, MUST NOT be greater than
        // > the value in zatoshi of block subsidy plus the transaction fees
        // > paid by transactions in this block.
        //
        // > If effectiveVersion ≥ 5 then this condition MUST hold:
        // > tx_out_count > 0 or nOutputsSapling > 0 or
        // > (nActionsOrchard > 0 and enableOutputsOrchard = 1).
        //
        // <https://zips.z.cash/protocol/protocol.pdf#txnconsensus>
        //
        // With shielded coinbase (ZIP 213), transparent outputs may be empty if there
        // are no funding streams.
        assert_ne!(
            outputs.len()
                + sapling_shielded_data
                    .as_ref()
                    .map_or(0, |data| data.outputs().count()),
            // + orchard_shielded_data
            0,
            "invalid coinbase transaction: must have at least one output"
        );

        let tx = Transaction::V6 {
            // > The transaction version number MUST be 4, 5, or 6. ...
            // > If the transaction version number is 6 then the version group ID
            // > MUST be 0x26A7270A.
            // > If effectiveVersion ≥ 5, the nConsensusBranchId field MUST match the consensus
            // > branch ID used for SIGHASH transaction hashes, as specified in [ZIP-244].
            network_upgrade: NetworkUpgrade::current(network, height),

            // There is no documented consensus rule for the lock time field in coinbase
            // transactions, so we just leave it unlocked. (We could also set it to `height`.)
            lock_time: LockTime::unlocked(),

            // > The nExpiryHeight field of a coinbase transaction MUST be equal to its
            // > block height.
            expiry_height: height,

            zip233_amount: zip233_amount.unwrap_or(Amount::zero()),

            inputs,
            outputs,

            // > In a version 6 coinbase transaction, the enableSpendsOrchard flag MUST be 0.
            // > In a version 6 transaction, the reserved bits 2 .. 7 of the flagsOrchard field
            // > MUST be zero.
            //
            // See the Zcash spec for additional shielded coinbase consensus rules.
            sapling_shielded_data,
            orchard_shielded_data,
        };

        assert_eq!(
            tx.has_sapling_shielded_data(),
            sapling_proven_bundle.is_some(),
        );

        #[cfg(feature = "shielded-mining-sapling")]
        let tx = update_coinbase_binding_sig(tx, network, height, sapling_proven_bundle);

        tx
    }

    /// Returns a new version 5 coinbase transaction for `network` and `height`,
    /// with the miner reward going to `miner_address` via `miner_pool`.
    pub fn new_v5_coinbase(
        network: &Network,
        height: Height,
        outputs: Vec<(Amount<NonNegative>, transparent::Script)>,
        miner_reward: Amount<NonNegative>,
        miner_pool: PoolType,
        miner_address: &Address,
        miner_data: Vec<u8>,
    ) -> Transaction {
        let (outputs, sapling_shielded_data, sapling_proven_bundle) =
            build_coinbase_outputs(miner_pool, miner_address, miner_reward, outputs);

        let orchard_shielded_data: Option<orchard::ShieldedData> = None;

        // # Consensus
        //
        // These consensus rules apply to v5 coinbase transactions after NU5 activation:
        //
        // > If effectiveVersion ≥ 5 then this condition MUST hold:
        // > tx_in_count > 0 or nSpendsSapling > 0 or
        // > (nActionsOrchard > 0 and enableSpendsOrchard = 1).
        //
        // > A coinbase transaction for a block at block height greater than 0 MUST have
        // > a script that, as its first item, encodes the block height as follows. ...
        // > let heightBytes be the signed little-endian representation of height,
        // > using the minimum nonzero number of bytes such that the most significant byte
        // > is < 0x80. The length of heightBytes MUST be in the range {1 .. 5}.
        // > Then the encoding is the length of heightBytes encoded as one byte,
        // > followed by heightBytes itself. This matches the encoding used by Bitcoin
        // > in the implementation of [BIP-34]
        // > (but the description here is to be considered normative).
        //
        // > A coinbase transaction script MUST have length in {2 .. 100} bytes.
        //
        // Zebra adds extra coinbase data if configured to do so.
        //
        // Since we're not using a lock time, any sequence number is valid here.
        // See `Transaction::lock_time()` for the relevant consensus rules.
        //
        // <https://zips.z.cash/protocol/protocol.pdf#txnconsensus>
        let inputs = vec![transparent::Input::new_coinbase(height, miner_data, None)];

        // > The block subsidy is composed of a miner subsidy and a series of funding streams.
        //
        // <https://zips.z.cash/protocol/protocol.pdf#subsidyconcepts>
        //
        // > The total value in zatoshi of transparent outputs from a coinbase transaction,
        // > minus vbalanceSapling, minus vbalanceOrchard, MUST NOT be greater than
        // > the value in zatoshi of block subsidy plus the transaction fees
        // > paid by transactions in this block.
        //
        // > If effectiveVersion ≥ 5 then this condition MUST hold:
        // > tx_out_count > 0 or nOutputsSapling > 0 or
        // > (nActionsOrchard > 0 and enableOutputsOrchard = 1).
        //
        // <https://zips.z.cash/protocol/protocol.pdf#txnconsensus>
        //
        // With shielded coinbase (ZIP 213), transparent outputs may be empty if there
        // are no funding streams.
        assert_ne!(
            outputs.len()
                + sapling_shielded_data
                    .as_ref()
                    .map_or(0, |data| data.outputs().count()),
            // orchard_shielded_data
            0,
            "invalid coinbase transaction: must have at least one output"
        );

        let tx = Transaction::V5 {
            // > The transaction version number MUST be 4 or 5. ...
            // > If the transaction version number is 5 then the version group ID
            // > MUST be 0x26A7270A.
            // > If effectiveVersion ≥ 5, the nConsensusBranchId field MUST match the consensus
            // > branch ID used for SIGHASH transaction hashes, as specified in [ZIP-244].
            network_upgrade: NetworkUpgrade::current(network, height),

            // There is no documented consensus rule for the lock time field in coinbase
            // transactions, so we just leave it unlocked. (We could also set it to `height`.)
            lock_time: LockTime::unlocked(),

            // > The nExpiryHeight field of a coinbase transaction MUST be equal to its
            // > block height.
            expiry_height: height,

            inputs,
            outputs,

            // > In a version 5 coinbase transaction, the enableSpendsOrchard flag MUST be 0.
            // > In a version 5 transaction, the reserved bits 2 .. 7 of the flagsOrchard field
            // > MUST be zero.
            //
            // See the Zcash spec for additional shielded coinbase consensus rules.
            sapling_shielded_data,
            orchard_shielded_data,
        };

        assert_eq!(
            tx.has_sapling_shielded_data(),
            sapling_proven_bundle.is_some(),
        );

        #[cfg(feature = "shielded-mining-sapling")]
        let tx = update_coinbase_binding_sig(tx, network, height, sapling_proven_bundle);

        tx
    }

    /// Returns a new version 4 coinbase transaction for `height`,
    /// with the miner reward going to `miner_address` via `miner_pool`.
    pub fn new_v4_coinbase(
        height: Height,
        outputs: Vec<(Amount<NonNegative>, transparent::Script)>,
        miner_pool: PoolType,
        miner_reward: Amount<NonNegative>,
        miner_address: &Address,
        miner_data: Vec<u8>,
    ) -> Transaction {
        // V4 coinbase only supports transparent miner pool
        assert_eq!(
            miner_pool,
            PoolType::Transparent,
            "v4 coinbase only supports transparent miner pool"
        );

        let (outputs, _, _) =
            build_coinbase_outputs(miner_pool, miner_address, miner_reward, outputs);

        // # Consensus
        //
        // See the other consensus rules above in new_v5_coinbase().
        //
        // > If effectiveVersion < 5, then at least one of tx_in_count, nSpendsSapling,
        // > and nJoinSplit MUST be nonzero.
        let inputs = vec![transparent::Input::new_coinbase(
            height,
            miner_data,
            // zcashd uses a sequence number of u32::MAX.
            Some(u32::MAX),
        )];

        // > If effectiveVersion < 5, then at least one of tx_out_count, nOutputsSapling,
        // > and nJoinSplit MUST be nonzero.
        assert!(
            !outputs.is_empty(),
            "invalid coinbase transaction: must have at least one output"
        );

        // > The transaction version number MUST be 4 or 5. ...
        // > If the transaction version number is 4 then the version group ID MUST be 0x892F2085.
        Transaction::V4 {
            lock_time: LockTime::unlocked(),
            expiry_height: height,
            inputs,
            outputs,
            joinsplit_data: None,
            sapling_shielded_data: None,
        }
    }
}

/// Builds coinbase outputs based on `miner_pool`.
///
/// For transparent mining, adds the miner reward to the funding outputs.
/// For Sapling mining, creates shielded data for the miner reward.
fn build_coinbase_outputs(
    miner_pool: PoolType,
    miner_address: &Address,
    miner_reward: Amount<NonNegative>,
    funding_outputs: Vec<(Amount<NonNegative>, transparent::Script)>,
) -> (
    Vec<transparent::Output>,
    Option<sapling::ShieldedData<sapling::SharedAnchor>>,
    Option<Bundle<InProgress<Proven, Unsigned>, i64>>,
) {
    let (outputs, sapling_shielded_data, sapling_proven_bundle) = match miner_pool {
        PoolType::Transparent => {
            let mut outputs = funding_outputs;
            outputs.insert(
                0,
                (
                    miner_reward,
                    transparent::Script::new(
                        &miner_address
                            .to_transparent_address()
                            .expect("address must have a transparent component")
                            .script()
                            .to_bytes(),
                    ),
                ),
            );
            (outputs, None, None)
        }
        #[cfg(feature = "shielded-mining-sapling")]
        PoolType::Shielded(ShieldedProtocol::Sapling) => {
            let (shielded_data, bundle) = sapling::coinbase::build_shielded_coinbase(
                miner_address
                    .to_sapling_address()
                    .expect("miner_address must support sapling"),
                miner_reward,
            );
            (funding_outputs, Some(shielded_data), Some(bundle))
        }
        #[cfg(not(feature = "shielded-mining-sapling"))]
        PoolType::Shielded(ShieldedProtocol::Sapling) => {
            unreachable!("shielded mining feature must be enabled")
        }
        PoolType::Shielded(ShieldedProtocol::Orchard) => {
            unimplemented!("zebrad doesn't support Orchard shielded coinbase")
        }
    };

    let outputs = outputs
        .into_iter()
        .map(|(amount, lock_script)| transparent::Output::new_coinbase(amount, lock_script))
        .collect();

    (outputs, sapling_shielded_data, sapling_proven_bundle)
}

/// Authorizes the Sapling binding signature for a coinbase transaction.
#[cfg(feature = "shielded-mining-sapling")]
fn update_coinbase_binding_sig(
    mut draft_tx: Transaction,
    network: &Network,
    height: Height,
    sapling_proven_bundle: Option<Bundle<InProgress<Proven, Unsigned>, i64>>,
) -> Transaction {
    let Some(sapling_proven_bundle) = sapling_proven_bundle else {
        return draft_tx;
    };

    // sighash for coinbase input has no hash type, no previous outputs, and no script code
    let sighash = draft_tx
        .sighash(
            NetworkUpgrade::current(network, height),
            HashType::NONE,
            Vec::new().into(),
            None,
        )
        .expect("draft coinbase should compute a sighash");

    let authorized_bundle = sapling_proven_bundle
        .apply_signatures(OsRng, sighash.0, &[])
        .expect("authorization should succeed");

    let draft_sapling_shielded_data = match &mut draft_tx {
        Transaction::V1 { .. } | Transaction::V2 { .. } | Transaction::V3 { .. } => {
            unreachable!("sapling binding signature is not supported for v1 to v3 transactions")
        }

        Transaction::V4 { .. } => {
            unimplemented!("zebrad doesn't support shielded coinbase for V4 transactions")
        }

        Transaction::V5 {
            sapling_shielded_data,
            ..
        } => sapling_shielded_data,

        #[cfg(all(zcash_unstable = "nu7", feature = "tx_v6"))]
        Transaction::V6 {
            sapling_shielded_data,
            ..
        } => sapling_shielded_data,
    }
    .as_mut()
    .expect("bundle indicates sapling shielded data is present");

    draft_sapling_shielded_data.binding_sig = authorized_bundle.authorization().binding_sig;

    draft_tx
}
