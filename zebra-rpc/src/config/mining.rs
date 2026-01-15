//! Mining config

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr, FromInto};

use zcash_address::ZcashAddress;
use zcash_protocol::{PoolType, ShieldedProtocol};

/// Mining configuration section.
#[serde_as]
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    /// Address for receiving miner subsidy and tx fees.
    ///
    /// Used in coinbase tx constructed in `getblocktemplate` RPC.
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub miner_address: Option<ZcashAddress>,

    /// The pool to use for mining rewards.
    ///
    /// Determines which receiver in the miner address receives the coinbase reward.
    /// The miner address must have a receiver for the selected pool type.
    ///
    /// Defaults to `"transparent"` if not specified. Valid values:
    /// - `"transparent"` - Mine to transparent address (default)
    /// - `"sapling"` - Mine to Sapling address (requires `shielded-mining-sapling` feature)
    /// - `"orchard"` - Mine to Orchard address (not yet implemented)
    #[serde_as(as = "Option<FromInto<MinerPoolType>>")]
    pub miner_pool: Option<PoolType>,

    // TODO: Internal miner config code was removed as part of https://github.com/ZcashFoundation/zebra/issues/8180
    // Find the removed code at https://github.com/ZcashFoundation/zebra/blob/v1.5.1/zebra-rpc/src/config/mining.rs#L18-L38
    // Restore the code when conditions are met. https://github.com/ZcashFoundation/zebra/issues/8183
    /// Extra data to include in coinbase transaction inputs.
    /// Limited to around 95 bytes by the consensus rules.
    ///
    /// If this string is hex-encoded, it will be hex-decoded into bytes.
    /// Otherwise, it will be UTF-8 encoded into bytes.
    pub extra_coinbase_data: Option<String>,

    /// Mine blocks using Zebra's internal miner, without an external mining pool or equihash solver.
    ///
    /// This experimental feature is only supported on regtest as it uses null solutions and skips checking
    /// for a valid Proof of Work.
    ///
    /// The internal miner is off by default.
    #[serde(default)]
    pub internal_miner: bool,
}

impl Config {
    /// Is the internal miner enabled using at least one thread?
    #[cfg(feature = "internal-miner")]
    pub fn is_internal_miner_enabled(&self) -> bool {
        // TODO: Changed to return always false so internal miner is never started. Part of https://github.com/ZcashFoundation/zebra/issues/8180
        // Find the removed code at https://github.com/ZcashFoundation/zebra/blob/v1.5.1/zebra-rpc/src/config/mining.rs#L83
        // Restore the code when conditions are met. https://github.com/ZcashFoundation/zebra/issues/8183
        self.internal_miner
    }
}

struct MinerPoolType(PoolType);

impl Serialize for MinerPoolType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self.0 {
            PoolType::Transparent => "transparent",
            PoolType::Shielded(pool) => match pool {
                ShieldedProtocol::Sapling => "sapling",
                ShieldedProtocol::Orchard => "orchard",
            },
        })
    }
}

impl<'de> Deserialize<'de> for MinerPoolType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "transparent" => Ok(MinerPoolType(PoolType::Transparent)),
            "sapling" => Ok(MinerPoolType(PoolType::Shielded(ShieldedProtocol::Sapling))),
            "orchard" => Ok(MinerPoolType(PoolType::Shielded(ShieldedProtocol::Orchard))),
            _ => Err(serde::de::Error::custom(format!("Invalid pool type: {s}"))),
        }
    }
}

impl From<PoolType> for MinerPoolType {
    fn from(pool: PoolType) -> Self {
        Self(pool)
    }
}

impl From<MinerPoolType> for PoolType {
    fn from(val: MinerPoolType) -> Self {
        val.0
    }
}
