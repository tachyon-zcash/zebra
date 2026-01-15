//! Tests for types and functions for the `getblocktemplate` RPC.

use zcash_keys::address::Address;
use zcash_protocol::PoolType;
use zcash_transparent::address::TransparentAddress;

use zebra_chain::{
    amount::Amount,
    block::Height,
    parameters::testnet::{self, ConfiguredActivationHeights, ConfiguredFundingStreams},
    serialization::{ZcashDeserializeInto, ZcashSerialize},
    transaction::Transaction,
};

use super::coinbase_outputs;

#[cfg(feature = "shielded-mining-sapling")]
use {
    super::generate_coinbase_and_roots,
    zcash_protocol::ShieldedProtocol,
    zebra_chain::{
        amount::NonNegative,
        parameters::{
            subsidy::{block_subsidy, funding_stream_values, miner_subsidy},
            Network,
        },
        primitives::zcash_note_encryption::decrypts_successfully,
    },
};

/// Tests that a minimal coinbase transaction can be generated.
#[test]
fn minimal_coinbase() -> Result<(), Box<dyn std::error::Error>> {
    let regtest = testnet::Parameters::build()
        .with_slow_start_interval(Height::MIN)
        .with_activation_heights(ConfiguredActivationHeights {
            nu6: Some(1),
            ..Default::default()
        })?
        .with_funding_streams(vec![ConfiguredFundingStreams {
            height_range: Some(Height(1)..Height(10)),
            recipients: None,
        }])
        .to_network()?;

    let miner_address = Address::from(TransparentAddress::PublicKeyHash([0x42; 20]));
    let height = Height(1);
    let miner_fee = Amount::zero();

    let (miner_reward, funding_outputs) = coinbase_outputs(&regtest, height, miner_fee);

    // It should be possible to generate a coinbase tx from these params.
    Transaction::new_v5_coinbase(
        &regtest,
        height,
        funding_outputs,
        miner_reward,
        PoolType::Transparent,
        &miner_address,
        vec![],
    )
    .zcash_serialize_to_vec()?
    // Deserialization contains checks for elementary consensus rules, which must pass.
    .zcash_deserialize_into::<Transaction>()?;

    Ok(())
}

/// Regtest Unified Address containing Sapling receiver for tests.
/// This is the same address used in zebrad/tests/common/config.rs.
#[cfg(feature = "shielded-mining-sapling")]
const REGTEST_MINER_ADDRESS: &str = "uregtest1a2yn922nnxyvnj4qmax07lkr7kmnyxq3rw0paa2kes87h2rapehrzgy8xrq665sg6aatmpgzkngwlumzr40e5y4vc40a809rsyqcwq25xfj5r2sxu774xdt6dj5xckjkv5ll0c2tv6qtsl60mpccwd6m95upy2da0rheqmkmxr7fv9z5uve0kpkmssxcuvzasewwns986yud6aact4y";

/// Creates a regtest network with required upgrades enabled at height 1.
#[cfg(feature = "shielded-mining-sapling")]
fn test_regtest_network() -> Network {
    use zebra_chain::parameters::testnet::RegtestParameters;

    Network::new_regtest(RegtestParameters {
        activation_heights: ConfiguredActivationHeights {
            heartwood: Some(1),
            canopy: Some(1),
            nu5: Some(1),
            ..Default::default()
        },
        funding_streams: Some(vec![ConfiguredFundingStreams {
            height_range: Some(Height(1)..Height(100)),
            recipients: None,
        }]),
        ..Default::default()
    })
}

/// Parses the test miner address for the given network.
#[cfg(feature = "shielded-mining-sapling")]
fn test_miner_address(network: &Network) -> Address {
    Address::try_from_zcash_address(
        network,
        REGTEST_MINER_ADDRESS
            .parse()
            .expect("hard-coded address is valid"),
    )
    .expect("address should be valid for network")
}

/// Tests basic Sapling shielded coinbase generation.
///
/// Verifies:
/// - Coinbase transaction has Sapling shielded data
/// - Value balance is negative (value INTO shielded pool)
/// - Exactly one Sapling output (miner reward)
#[test]
#[cfg(feature = "shielded-mining-sapling")]
fn sapling_coinbase_basic() {
    let network = test_regtest_network();
    let miner_address = test_miner_address(&network);
    let height = Height(1);

    let (coinbase_template, _roots) = generate_coinbase_and_roots(
        &network,
        height,
        PoolType::Shielded(ShieldedProtocol::Sapling),
        &miner_address,
        &[],
        None,
        vec![],
        #[cfg(all(zcash_unstable = "nu7", feature = "tx_v6"))]
        None,
    )
    .expect("coinbase generation should succeed");

    let tx: Transaction = coinbase_template
        .data()
        .as_ref()
        .zcash_deserialize_into()
        .expect("transaction should deserialize");

    // Verify coinbase has Sapling shielded data
    assert!(
        tx.has_sapling_shielded_data(),
        "coinbase transaction should have Sapling shielded data"
    );

    // Verify value_balance is negative (value flowing INTO shielded pool)
    let value_balance = tx.sapling_value_balance();
    let sapling_amount = value_balance.sapling_amount();
    assert!(
        sapling_amount < Amount::<zebra_chain::amount::NegativeAllowed>::zero(),
        "sapling value_balance should be negative (value into pool), got {:?}",
        sapling_amount
    );

    // Verify there is exactly one Sapling output (the miner reward)
    assert_eq!(
        tx.sapling_outputs().count(),
        1,
        "coinbase should have exactly one Sapling output for miner reward"
    );
}

/// Tests ZIP 213 OVK compliance for Sapling coinbase.
///
/// ZIP 213 requires shielded coinbase outputs be recoverable with
/// an all-zeros outgoing viewing key for supply auditability.
#[test]
#[cfg(feature = "shielded-mining-sapling")]
fn sapling_coinbase_zip213_ovk() {
    let network = test_regtest_network();
    let miner_address = test_miner_address(&network);
    let height = Height(1);

    let (coinbase_template, _roots) = generate_coinbase_and_roots(
        &network,
        height,
        PoolType::Shielded(ShieldedProtocol::Sapling),
        &miner_address,
        &[],
        None,
        vec![],
        #[cfg(all(zcash_unstable = "nu7", feature = "tx_v6"))]
        None,
    )
    .expect("coinbase generation should succeed");

    let tx: Transaction = coinbase_template
        .data()
        .as_ref()
        .zcash_deserialize_into()
        .expect("transaction should deserialize");

    assert!(
        tx.has_sapling_shielded_data(),
        "coinbase transaction should have Sapling shielded data"
    );

    // Verify Sapling outputs can be decrypted with all-zeros OVK per ZIP 213
    assert!(
        decrypts_successfully(&tx, &network, height),
        "coinbase outputs should decrypt with all-zeros OVK per ZIP 213"
    );
}

/// Tests value balance calculation for Sapling coinbase.
///
/// Verifies:
/// - Value balance is negative (value INTO shielded pool)
/// - |value_balance| equals miner reward
/// - transparent_outputs + |value_balance| = block_subsidy
#[test]
#[cfg(feature = "shielded-mining-sapling")]
fn sapling_coinbase_value_balance() {
    let network = test_regtest_network();
    let miner_address = test_miner_address(&network);
    let height = Height(1);

    let (coinbase_template, _roots) = generate_coinbase_and_roots(
        &network,
        height,
        PoolType::Shielded(ShieldedProtocol::Sapling),
        &miner_address,
        &[],
        None,
        vec![],
        #[cfg(all(zcash_unstable = "nu7", feature = "tx_v6"))]
        None,
    )
    .expect("coinbase generation should succeed");

    let tx: Transaction = coinbase_template
        .data()
        .as_ref()
        .zcash_deserialize_into()
        .expect("transaction should deserialize");

    // Get expected values
    let expected_block_subsidy =
        block_subsidy(height, &network).expect("block subsidy should be valid");
    let expected_miner_reward = miner_subsidy(height, &network, expected_block_subsidy)
        .expect("miner subsidy should be valid");

    // Get actual value_balance (negative = value INTO shielded pool)
    let value_balance = tx.sapling_value_balance();
    let sapling_amount = value_balance.sapling_amount();

    assert!(
        sapling_amount < Amount::<zebra_chain::amount::NegativeAllowed>::zero(),
        "value_balance should be negative for shielded coinbase"
    );

    // The absolute value of sapling_amount should equal miner reward
    let value_into_pool: Amount<NonNegative> = (-sapling_amount)
        .constrain()
        .expect("negated value_balance should be non-negative");

    assert_eq!(
        value_into_pool, expected_miner_reward,
        "value flowing into shielded pool should equal miner reward"
    );

    // Calculate total transparent output value (funding streams)
    let transparent_output_total: Amount<NonNegative> = tx
        .outputs()
        .iter()
        .map(|o| o.value())
        .sum::<Result<Amount<NonNegative>, _>>()
        .expect("transparent output sum should be valid");

    // Verify: transparent_outputs + value_into_pool = block_subsidy
    let total_value =
        (transparent_output_total + value_into_pool).expect("total value should not overflow");

    assert_eq!(
        total_value, expected_block_subsidy,
        "transparent outputs + shielded value should equal block subsidy"
    );
}

/// Tests that funding streams remain transparent with Sapling coinbase.
///
/// ZIP 213 requires only the miner reward go to shielded pool.
/// Funding stream outputs must remain transparent.
#[test]
#[cfg(feature = "shielded-mining-sapling")]
fn sapling_coinbase_funding_streams_transparent() {
    let network = test_regtest_network();
    let miner_address = test_miner_address(&network);
    let height = Height(1);

    let (coinbase_template, _roots) = generate_coinbase_and_roots(
        &network,
        height,
        PoolType::Shielded(ShieldedProtocol::Sapling),
        &miner_address,
        &[],
        None,
        vec![],
        #[cfg(all(zcash_unstable = "nu7", feature = "tx_v6"))]
        None,
    )
    .expect("coinbase generation should succeed");

    let tx: Transaction = coinbase_template
        .data()
        .as_ref()
        .zcash_deserialize_into()
        .expect("transaction should deserialize");

    // Get expected funding stream values
    let expected_block_subsidy =
        block_subsidy(height, &network).expect("block subsidy should be valid");
    let funding_streams = funding_stream_values(height, &network, expected_block_subsidy)
        .expect("funding stream values should be valid");

    // Filter out lockbox (deferred) streams - they don't create outputs
    let transparent_funding_streams: Vec<_> = funding_streams
        .iter()
        .filter(|(receiver, _)| {
            !matches!(
                receiver,
                zebra_chain::parameters::subsidy::FundingStreamReceiver::Deferred
            )
        })
        .collect();

    // Verify transparent outputs match funding stream count
    assert_eq!(
        tx.outputs().len(),
        transparent_funding_streams.len(),
        "transparent outputs should match number of non-deferred funding streams"
    );

    // Verify each funding stream amount appears in transparent outputs
    let transparent_values: Vec<Amount<NonNegative>> =
        tx.outputs().iter().map(|o| o.value()).collect();

    for (receiver, expected_amount) in &transparent_funding_streams {
        assert!(
            transparent_values.contains(expected_amount),
            "funding stream {:?} with amount {:?} should appear in transparent outputs",
            receiver,
            expected_amount
        );
    }

    // Verify only ONE Sapling output (the miner reward)
    assert_eq!(
        tx.sapling_outputs().count(),
        1,
        "only miner reward should be shielded, funding streams should be transparent"
    );
}
