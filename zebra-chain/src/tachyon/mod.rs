#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ShieldedTransactionAggregate {}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Tachygram {}

#[cfg(any(test, feature = "proptest-impl"))]
pub mod arbitrary;