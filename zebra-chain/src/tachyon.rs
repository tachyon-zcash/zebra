//! Tachyon shielded pool chain-state types.
//!
//! Tachyon has no note commitment tree. Its pool state is a running *anchor*: a
//! Poseidon hash sequence that absorbs each proof stamp's tachygram-set
//! commitment and lifts at epoch boundaries. The anchor takes the role that a
//! note commitment tree root has for the Sapling, Orchard, and Ironwood pools.

use std::{fmt, io};

use serde::{Deserialize, Serialize};

use crate::{
    block::Block,
    parameters::{Network, NetworkUpgrade},
    serialization::{ReadZcashExt, SerializationError, ZcashDeserialize, ZcashSerialize},
};

/// The number of blocks in a Tachyon epoch, from [`zcash_tachyon::constants::EPOCH_SIZE`].
///
/// Tachygrams are kept in the epoch's working set from the epoch's first block, and pruned when
/// the epoch ends; the anchor fold lifts into each new epoch at its first block. Epochs are
/// indexed by pool height: epoch `E` spans pool heights `[E * EPOCH_LENGTH, (E+1) * EPOCH_LENGTH)`.
///
/// This re-export keeps Zebra's epoch math in one place, agreeing with the tachyon crate's own
/// epoch helpers. Note that `zcash_tachyon` shrinks the constant under its *own* `cfg(test)`
/// builds; that never applies here, because dependencies are always compiled without
/// `cfg(test)`.
pub const EPOCH_LENGTH: u32 = zcash_tachyon::constants::EPOCH_SIZE;

/// The 0-based Tachyon pool height of `height`: its offset above the ZFuture
/// activation height.
///
/// Returns `None` if ZFuture is not active at `height`, or has no activation
/// height on `network`. The pool fold (and its epoch schedule) is indexed by
/// pool height, not chain height.
pub fn pool_height(network: &Network, height: crate::block::Height) -> Option<u32> {
    let activation_height = NetworkUpgrade::ZFuture.activation_height(network)?;
    height.0.checked_sub(activation_height.0)
}

/// The Tachyon epoch index containing `pool_height`.
pub fn epoch_of_pool_height(pool_height: u32) -> u32 {
    pool_height / EPOCH_LENGTH
}

/// Whether `pool_height` is the first block of its epoch.
pub fn is_epoch_first(pool_height: u32) -> bool {
    pool_height.is_multiple_of(EPOCH_LENGTH)
}

/// The Tachyon epoch index containing the block at `height`, if the pool has
/// started by then on `network`.
pub fn epoch(network: &Network, height: crate::block::Height) -> Option<u32> {
    pool_height(network, height).map(epoch_of_pool_height)
}

/// Whether the block at `earlier` falls within the two-epoch consensus scan window of the
/// block at `later`: `later`'s own epoch or the immediately preceding one.
///
/// Consensus rejects a stamp tachygram already revealed in this window, and requires a
/// stamp's anchor to come from a block in it. The window spans *two* epochs because a
/// spend publishes a nullifier pair — one for its anchor epoch `e` and one for epoch
/// `e + 1` — and the next-epoch nullifier is exactly the tachygram a repeat spend of the
/// same note would have to publish in epoch `e + 1`; scanning the preceding epoch catches
/// that collision. Beyond the window, nullifiers are epoch-flavored (a fresh GGM leaf per
/// epoch), so a repeated tachygram would imply a Poseidon collision: consensus neither
/// tracks nor rejects it.
///
/// See the "Tachygrams" chapter of the tachyon book (`Consensus validation`).
///
/// Returns `false` if the pool has not started at either height.
pub fn within_scan_window(
    network: &Network,
    earlier: crate::block::Height,
    later: crate::block::Height,
) -> bool {
    match (epoch(network, earlier), epoch(network, later)) {
        (Some(earlier_epoch), Some(later_epoch)) => earlier_epoch + 1 >= later_epoch,
        _ => false,
    }
}

/// The running anchor of the Tachyon pool after some block.
///
/// Stored as the canonical 32-byte encoding of the underlying Pallas base field
/// element ([`zcash_tachyon::Anchor`]'s wire encoding).
///
/// The [`Default`] value (all-zero bytes) encodes the field zero element: the
/// fold seed of a pool that has not started yet. It is *not* the anchor of any
/// block; the first pool block's epoch lift turns it into tachyon's genesis
/// anchor. Pre-ZFuture blocks use the default value as an ignored placeholder in
/// chain-history leaves.
#[derive(Clone, Copy, Default, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Anchor(pub [u8; 32]);

impl fmt::Debug for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("tachyon::Anchor")
            .field(&hex::encode(self.0))
            .finish()
    }
}

impl From<[u8; 32]> for Anchor {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<Anchor> for [u8; 32] {
    fn from(anchor: Anchor) -> Self {
        anchor.0
    }
}

impl From<&Anchor> for [u8; 32] {
    fn from(anchor: &Anchor) -> Self {
        anchor.0
    }
}

impl ZcashSerialize for Anchor {
    fn zcash_serialize<W: io::Write>(&self, mut writer: W) -> Result<(), io::Error> {
        writer.write_all(&self.0)
    }
}

impl ZcashDeserialize for Anchor {
    fn zcash_deserialize<R: io::Read>(mut reader: R) -> Result<Self, SerializationError> {
        Ok(Self(reader.read_32_bytes()?))
    }
}

impl From<zcash_tachyon::Anchor> for Anchor {
    fn from(anchor: zcash_tachyon::Anchor) -> Self {
        let mut bytes = Vec::with_capacity(32);
        anchor
            .write(&mut bytes)
            .expect("write to Vec is infallible");
        Self(
            bytes
                .try_into()
                .expect("tachyon anchors always encode as 32 bytes"),
        )
    }
}

/// A tachygram: a nullifier or note commitment revealed by a Tachyon proof stamp.
///
/// Stored as the canonical 32-byte encoding of the underlying Pallas base field element.
/// Tachygrams are tracked in a two-epoch working set: revealing a tachygram already revealed
/// in the block's own epoch or the immediately preceding one is invalid (see
/// [`within_scan_window`]).
#[derive(Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Tachygram(pub [u8; 32]);

impl fmt::Debug for Tachygram {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("tachyon::Tachygram")
            .field(&hex::encode(self.0))
            .finish()
    }
}

impl From<[u8; 32]> for Tachygram {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl From<Tachygram> for [u8; 32] {
    fn from(tachygram: Tachygram) -> Self {
        tachygram.0
    }
}

impl From<&Tachygram> for [u8; 32] {
    fn from(tachygram: &Tachygram) -> Self {
        tachygram.0
    }
}

impl From<zcash_tachyon::Tachygram> for Tachygram {
    fn from(tachygram: zcash_tachyon::Tachygram) -> Self {
        // `Tachygram` newtypes a Pallas base field element; its canonical encoding is the
        // element's 32-byte representation.
        let fp: halo2::pasta::pallas::Base = tachygram.into();
        Self(fp.into())
    }
}

impl Anchor {
    /// Convert to the tachyon crate's anchor type.
    ///
    /// # Panics
    ///
    /// Panics if the bytes are not the canonical encoding of a field element.
    /// Anchors are only constructed from canonical encodings (the fold in
    /// [`Self::advance_with_block`], or Zebra's own storage of its results),
    /// so this can only happen on state corruption.
    fn to_tachyon(self) -> zcash_tachyon::Anchor {
        zcash_tachyon::Anchor::read(&self.0[..])
            .expect("stored tachyon anchors are canonical field element encodings")
    }

    /// Compute the Tachyon pool anchor after `block`.
    ///
    /// `self` is the anchor after the parent block, or [`Anchor::default`]
    /// when `block` is the first pool block (ZFuture activation). `pool_height` is
    /// the block's 0-based height above the ZFuture activation height, which
    /// drives the epoch schedule (see [`EPOCH_LENGTH`]).
    ///
    /// This mirrors tachyon's reference pool fold: the first pool block starts
    /// at the genesis anchor, later epoch-first pool heights lift the anchor
    /// into their epoch, and each proof stamp's tachygram-set commitment is
    /// absorbed in transaction order. Adjunct (pointer-stamped) bundles do not
    /// contribute: their tachygrams are carried by the aggregate's proof stamp.
    ///
    /// The returned [`AnchorAdvance`] also carries the epoch-boundary anchor
    /// when `block` starts a new epoch: the epoch's initial state, which spend
    /// lineages root at, tracked in state per epoch.
    ///
    /// # Errors
    ///
    /// Returns an error if an epoch or stamp would produce an invalid anchor
    /// transition.
    pub fn advance_with_block(
        &self,
        pool_height: u32,
        block: &Block,
    ) -> Result<AnchorAdvance, zcash_tachyon::AnchorError> {
        use zcash_tachyon::TachyonBundle;

        let epoch = zcash_tachyon::EpochIndex(epoch_of_pool_height(pool_height));

        let mut anchor = if pool_height == 0 {
            zcash_tachyon::Anchor::default()
        } else {
            self.to_tachyon()
        };
        let epoch_boundary = if pool_height == 0 {
            Some(Anchor::from(anchor))
        } else if is_epoch_first(pool_height) {
            anchor = anchor.next_epoch(epoch)?;
            Some(Anchor::from(anchor))
        } else {
            None
        };

        for transaction in &block.transactions {
            let Some(shielded_data) = transaction.tachyon_shielded_data() else {
                continue;
            };
            if let TachyonBundle::Proven(bundle) = &shielded_data.0 {
                // The stamp carries its tachygram-set commitment; semantic
                // verification confirms it against a valid, non-empty set of
                // tachygrams (`ProofStamp::is_accumulating`).
                anchor = anchor.next_stamp(epoch, &bundle.stamp.tachygram_set)?;
            }
        }

        Ok(AnchorAdvance {
            post_block: Anchor::from(anchor),
            epoch_boundary,
        })
    }
}

/// The result of advancing the Tachyon pool anchor with one block.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnchorAdvance {
    /// The pool anchor after the block.
    pub post_block: Anchor,

    /// The epoch-boundary anchor, if the block is the first of a new epoch:
    /// the anchor after the epoch lift, before any of the block's stamps are
    /// absorbed. `None` for mid-epoch blocks.
    pub epoch_boundary: Option<Anchor>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_serialization_round_trip() {
        let anchor = Anchor([42; 32]);
        let mut bytes = Vec::new();
        anchor.zcash_serialize(&mut bytes).unwrap();
        assert_eq!(bytes.len(), 32);
        assert_eq!(Anchor::zcash_deserialize(&bytes[..]).unwrap(), anchor);
    }

    #[test]
    fn epoch_helpers_follow_epoch_length() {
        assert_eq!(epoch_of_pool_height(0), 0);
        assert_eq!(epoch_of_pool_height(EPOCH_LENGTH - 1), 0);
        assert_eq!(epoch_of_pool_height(EPOCH_LENGTH), 1);
        assert_eq!(epoch_of_pool_height(3 * EPOCH_LENGTH + 7), 3);

        assert!(is_epoch_first(0));
        assert!(!is_epoch_first(1));
        assert!(!is_epoch_first(EPOCH_LENGTH - 1));
        assert!(is_epoch_first(EPOCH_LENGTH));
        assert!(is_epoch_first(2 * EPOCH_LENGTH));
    }

    mod fold {
        use std::sync::Arc;

        use super::super::*;
        use crate::{block::Block, serialization::ZcashDeserialize};

        /// A block with no tachyon bundles (any pre-ZFuture vector block works).
        fn stampless_block() -> Block {
            Block::zcash_deserialize(&zebra_test::vectors::BLOCK_MAINNET_419200_BYTES[..])
                .expect("block test vector is valid")
        }

        /// The first pool block starts at tachyon's genesis anchor.
        #[test]
        fn first_pool_block_uses_tachyon_genesis_anchor() {
            let block = Arc::new(stampless_block());
            let advance = Anchor::default().advance_with_block(0, &block).unwrap();
            let expected = Anchor::from(zcash_tachyon::Anchor::default());

            assert_eq!(advance.post_block, expected);
            assert_eq!(advance.epoch_boundary, Some(expected));
        }

        #[test]
        fn tachyon_anchor_conversion_round_trips() {
            let anchor = zcash_tachyon::Anchor::default();
            assert_eq!(Anchor::from(anchor).to_tachyon(), anchor);
        }

        /// Zebra's epoch helpers agree with the tachyon crate's own epoch math.
        #[test]
        fn epoch_helpers_agree_with_tachyon_crate() {
            for pool_height in [
                0,
                1,
                EPOCH_LENGTH - 1,
                EPOCH_LENGTH,
                EPOCH_LENGTH + 1,
                3 * EPOCH_LENGTH + 7,
            ] {
                let crate_height = zcash_tachyon::BlockHeight(pool_height);
                assert_eq!(
                    epoch_of_pool_height(pool_height),
                    crate_height.epoch().0,
                    "epoch mismatch at pool height {pool_height}",
                );
                assert_eq!(
                    is_epoch_first(pool_height),
                    crate_height.is_epoch_first(),
                    "epoch-first mismatch at pool height {pool_height}",
                );
            }
        }

        /// The per-block fold matches tachyon's reference semantics for
        /// stamp-less blocks: the anchor changes only at epoch boundaries.
        /// Boundary lifts also surface the epoch anchor.
        #[test]
        fn stampless_fold_matches_reference() {
            let block = Arc::new(stampless_block());

            // Pool block 0 starts at tachyon's genesis anchor.
            let genesis_advance = Anchor::default().advance_with_block(0, &block).unwrap();
            let expected = zcash_tachyon::Anchor::default();
            assert_eq!(genesis_advance.post_block, Anchor::from(expected));
            assert_eq!(
                genesis_advance.epoch_boundary,
                Some(Anchor::from(zcash_tachyon::Anchor::default())),
            );

            // A stamp-less mid-epoch block leaves the anchor unchanged.
            let next_advance = genesis_advance
                .post_block
                .advance_with_block(1, &block)
                .unwrap();
            assert_eq!(next_advance.post_block, Anchor::from(expected));
            assert_eq!(next_advance.epoch_boundary, None);

            // An epoch-first pool height lifts into its epoch and surfaces the
            // boundary anchor.
            let boundary_advance = next_advance
                .post_block
                .advance_with_block(EPOCH_LENGTH, &block)
                .unwrap();
            let expected_lift = expected.next_epoch(zcash_tachyon::EpochIndex(1)).unwrap();
            assert_eq!(
                boundary_advance.epoch_boundary,
                Some(Anchor::from(expected_lift)),
            );
            assert_eq!(boundary_advance.post_block, Anchor::from(expected_lift));
        }
    }
}
