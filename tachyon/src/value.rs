//! Tachyon value types.

use crate::primitives::PallasPoint;

/// A value commitment for a Tachyon action.
///
/// Commits to the value being transferred in an action without revealing it.
/// This is a Pedersen commitment (curve point) used in value balance verification.
///
/// The commitment has the form: `[v] V + [rcv] R` where:
/// - `v` is the value
/// - `rcv` is the randomness
/// - `V` and `R` are generator points
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValueCommitment(pub PallasPoint);
