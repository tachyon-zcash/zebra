//! Blocks and block-related structures (heights, headers, etc.)

use std::{collections::HashMap, fmt, ops::Neg, sync::Arc};

use halo2::pasta::pallas;

use crate::{
    amount::{DeferredPoolBalanceChange, NegativeAllowed},
    block::merkle::AuthDataRoot,
    fmt::DisplayToDebug,
    orchard,
    parameters::{Network, NetworkUpgrade},
    sapling,
    serialization::{TrustedPreallocate, MAX_PROTOCOL_MESSAGE_LEN},
    sprout,
    tachyon,
    transaction::Transaction,
    transparent,
    value_balance::{ValueBalance, ValueBalanceError},
};

mod commitment;
mod error;
mod hash;
mod header;
mod height;
mod serialize;

pub mod genesis;
pub mod merkle;

#[cfg(any(test, feature = "proptest-impl"))]
pub mod arbitrary;
#[cfg(any(test, feature = "bench", feature = "proptest-impl"))]
pub mod tests;

pub use commitment::{
    ChainHistoryBlockTxAuthCommitmentHash, ChainHistoryMmrRootHash, Commitment, CommitmentError,
    CHAIN_HISTORY_ACTIVATION_RESERVED,
};
pub use hash::Hash;
pub use header::{BlockTimeError, CountedHeader, Header, ZCASH_BLOCK_VERSION};
pub use height::{Height, HeightDiff, TryIntoHeight};
pub use serialize::{SerializedBlock, MAX_BLOCK_BYTES};

#[cfg(any(test, feature = "proptest-impl"))]
pub use arbitrary::LedgerState;

/// A Zcash block, containing a header and a list of transactions.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    any(test, feature = "proptest-impl", feature = "elasticsearch"),
    derive(Serialize)
)]
pub struct Block {
    /// The block header, containing block metadata.
    pub header: Arc<Header>,
    /// The block transactions.
    pub transactions: Vec<Arc<Transaction>>,
    /// The block tachygrams
    pub tachygrams: Option<Vec<tachyon::Tachygram>>,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fmter = f.debug_struct("Block");

        if let Some(height) = self.coinbase_height() {
            fmter.field("height", &height);
        }
        fmter.field("transactions", &self.transactions.len());
        fmter.field("hash", &DisplayToDebug(self.hash()));

        fmter.finish()
    }
}

impl Block {
    /// Return the block height reported in the coinbase transaction, if any.
    ///
    /// Note
    ///
    /// Verified blocks have a valid height.
    pub fn coinbase_height(&self) -> Option<Height> {
        self.transactions
            .first()
            .and_then(|tx| tx.inputs().first())
            .and_then(|input| match input {
                transparent::Input::Coinbase { ref height, .. } => Some(*height),
                _ => None,
            })
    }

    /// Compute the hash of this block.
    pub fn hash(&self) -> Hash {
        Hash::from(self)
    }

    /// Get the parsed block [`Commitment`] for this block.
    ///
    /// The interpretation of the commitment depends on the
    /// configured `network`, and this block's height.
    ///
    /// Returns an error if this block does not have a block height,
    /// or if the commitment value is structurally invalid.
    pub fn commitment(&self, network: &Network) -> Result<Commitment, CommitmentError> {
        match self.coinbase_height() {
            None => Err(CommitmentError::MissingBlockHeight {
                block_hash: self.hash(),
            }),
            Some(height) => Commitment::from_bytes(*self.header.commitment_bytes, network, height),
        }
    }

    /// Check if the `network_upgrade` fields from each transaction in the block matches
    /// the network upgrade calculated from the `network` and block height.
    ///
    /// # Consensus
    ///
    /// > [NU5 onward] The nConsensusBranchId field MUST match the consensus branch ID used
    /// > for SIGHASH transaction hashes, as specified in [ZIP-244].
    ///
    /// <https://zips.z.cash/protocol/protocol.pdf#txnconsensus>
    ///
    /// [ZIP-244]: https://zips.z.cash/zip-0244
    #[allow(clippy::unwrap_in_result)]
    pub fn check_transaction_network_upgrade_consistency(
        &self,
        network: &Network,
    ) -> Result<(), error::BlockError> {
        let block_nu =
            NetworkUpgrade::current(network, self.coinbase_height().expect("a valid height"));

        if self
            .transactions
            .iter()
            .filter_map(|trans| trans.as_ref().network_upgrade())
            .any(|trans_nu| trans_nu != block_nu)
        {
            return Err(error::BlockError::WrongTransactionConsensusBranchId);
        }

        Ok(())
    }

    /// Access the [`sprout::Nullifier`]s from all transactions in this block.
    pub fn sprout_nullifiers(&self) -> impl Iterator<Item = &sprout::Nullifier> {
        self.transactions
            .iter()
            .flat_map(|transaction| transaction.sprout_nullifiers())
    }

    /// Access the [`sapling::Nullifier`]s from all transactions in this block.
    pub fn sapling_nullifiers(&self) -> impl Iterator<Item = &sapling::Nullifier> {
        self.transactions
            .iter()
            .flat_map(|transaction| transaction.sapling_nullifiers())
    }

    /// Access the [`orchard::Nullifier`]s from all transactions in this block.
    pub fn orchard_nullifiers(&self) -> impl Iterator<Item = &orchard::Nullifier> {
        self.transactions
            .iter()
            .flat_map(|transaction| transaction.orchard_nullifiers())
    }

    /// Access the [`sprout::NoteCommitment`]s from all transactions in this block.
    pub fn sprout_note_commitments(&self) -> impl Iterator<Item = &sprout::NoteCommitment> {
        self.transactions
            .iter()
            .flat_map(|transaction| transaction.sprout_note_commitments())
    }

    /// Access the [sapling note commitments](`sapling_crypto::note::ExtractedNoteCommitment`)
    /// from all transactions in this block.
    pub fn sapling_note_commitments(
        &self,
    ) -> impl Iterator<Item = &sapling_crypto::note::ExtractedNoteCommitment> {
        self.transactions
            .iter()
            .flat_map(|transaction| transaction.sapling_note_commitments())
    }

    /// Access the [orchard note commitments](pallas::Base) from all transactions in this block.
    pub fn orchard_note_commitments(&self) -> impl Iterator<Item = &pallas::Base> {
        self.transactions
            .iter()
            .flat_map(|transaction| transaction.orchard_note_commitments())
    }

    /// Count how many Sapling transactions exist in a block,
    /// i.e. transactions "where either of vSpendsSapling or vOutputsSapling is non-empty"
    /// <https://zips.z.cash/zip-0221#tree-node-specification>.
    pub fn sapling_transactions_count(&self) -> u64 {
        self.transactions
            .iter()
            .filter(|tx| tx.has_sapling_shielded_data())
            .count()
            .try_into()
            .expect("number of transactions must fit u64")
    }

    /// Count how many Orchard transactions exist in a block,
    /// i.e. transactions "where vActionsOrchard is non-empty."
    /// <https://zips.z.cash/zip-0221#tree-node-specification>.
    pub fn orchard_transactions_count(&self) -> u64 {
        self.transactions
            .iter()
            .filter(|tx| tx.has_orchard_shielded_data())
            .count()
            .try_into()
            .expect("number of transactions must fit u64")
    }

    /// Returns the overall chain value pool change in this block---the negative sum of the
    /// transaction value balances in this block.
    ///
    /// These are the changes in the transparent, Sprout, Sapling, Orchard, and
    /// Deferred chain value pools, as a result of this block.
    ///
    /// Positive values are added to the corresponding chain value pool and negative values are
    /// removed from the corresponding pool.
    ///
    /// <https://zebra.zfnd.org/dev/rfcs/0012-value-pools.html#definitions>
    ///
    /// The given `utxos` must contain the [`transparent::Utxo`]s of every input in this block,
    /// including UTXOs created by earlier transactions in this block. It can also contain unrelated
    /// UTXOs, which are ignored.
    ///
    /// Note that the chain value pool has the opposite sign to the transaction value pool.
    pub fn chain_value_pool_change(
        &self,
        utxos: &HashMap<transparent::OutPoint, transparent::Utxo>,
        deferred_pool_balance_change: Option<DeferredPoolBalanceChange>,
    ) -> Result<ValueBalance<NegativeAllowed>, ValueBalanceError> {
        Ok(*self
            .transactions
            .iter()
            .flat_map(|t| t.value_balance(utxos))
            .sum::<Result<ValueBalance<NegativeAllowed>, _>>()?
            .neg()
            .set_deferred_amount(
                deferred_pool_balance_change
                    .map(DeferredPoolBalanceChange::value)
                    .unwrap_or_default(),
            ))
    }

    /// Compute the root of the authorizing data Merkle tree,
    /// as defined in [ZIP-244].
    ///
    /// [ZIP-244]: https://zips.z.cash/zip-0244
    pub fn auth_data_root(&self) -> AuthDataRoot {
        self.transactions.iter().collect::<AuthDataRoot>()
    }

    /// Compute the root of the block tachygram Merkle tree.
    ///
    /// The tree structure is:
    /// - First leaf: `previous_block_tachygram_root` (from the previous block's header)
    /// - Remaining leaves: tachygrams from this block's `tachygrams` field
    ///
    /// Returns the computed root that should match the `block_tachygram_root` field
    /// in this block's header.
    pub fn compute_block_tachygram_root(
        &self,
        previous_block_tachygram_root: orchard::tree::Root,
    ) -> orchard::tree::Root {
        // Collect all leaves: previous root + tachygrams
        let mut leaves: Vec<pallas::Base> = Vec::new();

        // First leaf is the previous block's tachygram root
        leaves.push(previous_block_tachygram_root.into());

        // Add leaves for each tachygram in this block
        if let Some(ref tachygrams) = self.tachygrams {
            for tachygram in tachygrams {
                leaves.push(Self::hash_tachygram_to_base(tachygram));
            }
        }

        // Build the merkle tree from the leaves
        Self::compute_merkle_root(&leaves)
    }

    /// Hash a tachygram to a pallas::Base field element.
    ///
    /// Uses BLAKE2b-256 personalized with "ZcashTachygramH" to hash the
    /// serialized tachygram, then interprets the result as a field element.
    fn hash_tachygram_to_base(_tachygram: &tachyon::Tachygram) -> pallas::Base {
        use halo2::pasta::group::ff::PrimeField;

        // Serialize the tachygram
        // For now, since Tachygram is empty, this will be empty bytes
        // TODO: Update this when Tachygram has fields
        let serialized = Vec::new(); // tachygram serialization would go here

        // Hash the serialized tachygram
        let hash = blake2b_simd::Params::new()
            .hash_length(32)
            .personal(b"ZcashTachygramH")
            .to_state()
            .update(&serialized)
            .finalize();

        // Convert the hash to a field element
        // We use from_repr which may fail if the value is not in the field,
        // but BLAKE2b output should be uniformly distributed
        let bytes: [u8; 32] = hash.as_bytes().try_into().expect("blake2b outputs 32 bytes");
        pallas::Base::from_repr(bytes).unwrap_or(pallas::Base::zero())
    }

    /// Compute the merkle root from a list of leaf nodes using the Orchard
    /// merkle hash function.
    ///
    /// Uses the same hash function as the Orchard note commitment tree
    /// (MerkleCRH^Orchard with Sinsemilla hash).
    fn compute_merkle_root(leaves: &[pallas::Base]) -> orchard::tree::Root {
        if leaves.is_empty() {
            return orchard::tree::Root::default();
        }

        if leaves.len() == 1 {
            return orchard::tree::Root::from(leaves[0]);
        }

        // Build the tree bottom-up
        let mut current_level = leaves.to_vec();
        let mut layer: u8 = Self::compute_tree_depth(leaves.len()) - 1;

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                let left = chunk[0];
                let right = if chunk.len() == 2 {
                    chunk[1]
                } else {
                    // If odd number of nodes, duplicate the last one
                    chunk[0]
                };

                // Use the Orchard merkle hash function
                next_level.push(Self::merkle_hash_orchard(layer, left, right));
            }

            current_level = next_level;
            layer = layer.saturating_sub(1);
        }

        orchard::tree::Root::from(current_level[0])
    }

    /// Compute the depth of a merkle tree needed to hold `leaf_count` leaves.
    fn compute_tree_depth(leaf_count: usize) -> u8 {
        if leaf_count <= 1 {
            return 0;
        }
        // Depth is ceil(log2(leaf_count))
        (leaf_count.next_power_of_two().trailing_zeros() as u8)
            .max(1)
    }

    /// MerkleCRH^Orchard Hash Function
    ///
    /// This is the same hash function used in the Orchard note commitment tree.
    /// Uses SinsemillaHash to combine two pallas::Base values at a given layer.
    fn merkle_hash_orchard(layer: u8, left: pallas::Base, right: pallas::Base) -> pallas::Base {
        use bitvec::prelude::*;
        use halo2::pasta::group::ff::PrimeField;

        const MERKLE_DEPTH: u8 = 32;

        let mut s = bitvec![u8, Lsb0;];

        // Prefix: l = I2LEBSP_10(MerkleDepth - 1 - layer)
        let l = MERKLE_DEPTH - 1 - layer;
        s.extend_from_bitslice(&BitArray::<_, Lsb0>::from([l, 0])[0..10]);
        s.extend_from_bitslice(&BitArray::<_, Lsb0>::from(left.to_repr())[0..255]);
        s.extend_from_bitslice(&BitArray::<_, Lsb0>::from(right.to_repr())[0..255]);

        match orchard::sinsemilla::sinsemilla_hash(b"z.cash:Orchard-MerkleCRH", &s) {
            Some(h) => h,
            None => pallas::Base::zero(),
        }
    }
}

impl<'a> From<&'a Block> for Hash {
    fn from(block: &'a Block) -> Hash {
        block.header.as_ref().into()
    }
}

/// A serialized Block hash takes 32 bytes
const BLOCK_HASH_SIZE: u64 = 32;

/// The maximum number of hashes in a valid Zcash protocol message.
impl TrustedPreallocate for Hash {
    fn max_allocation() -> u64 {
        // Every vector type requires a length field of at least one byte for de/serialization.
        // Since a block::Hash takes 32 bytes, we can never receive more than (MAX_PROTOCOL_MESSAGE_LEN - 1) / 32 hashes in a single message
        ((MAX_PROTOCOL_MESSAGE_LEN - 1) as u64) / BLOCK_HASH_SIZE
    }
}
