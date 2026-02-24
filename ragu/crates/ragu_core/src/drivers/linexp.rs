use ff::Field;

use super::Coeff;

/// Linear expressions represent accumulated linear combinations of wires. They
/// provide an efficient interface for adding or subtracting terms, allowing
/// drivers to optimize arithmetic depending on the coefficient, wire type and
/// context.
///
/// In Ragu, linear expressions cannot be directly scaled, since scaling
/// arbitrary combinations can be inefficient in some contexts. Instead, each
/// expression maintains a "gain" factor (initialized to $1$), and every term
/// added is multiplied by the _current_ gain. The gain can be updated at any
/// time, affecting only subsequent terms. This is equivalent to scale-and-add
/// techniques, though it can be more awkward or unfamiliar.
pub trait LinearExpression<W: Clone, F: Field>: Sized {
    /// This adds a term to the linear expression, described by a wire and an
    /// associated coefficient. Terms being added are always scaled by the
    /// current gain.
    fn add_term(self, wire: &W, coeff: Coeff<F>) -> Self;

    /// Scale the current gain by some amount.
    fn gain(self, coeff: Coeff<F>) -> Self;

    /// Extends the linear expression using an iterator of terms.
    fn extend(mut self, with: impl IntoIterator<Item = (W, Coeff<F>)>) -> Self {
        for (wire, coeff) in with {
            self = self.add_term(&wire, coeff);
        }
        self
    }

    /// Adds a wire to the linear expression with a coefficient of $1$.
    fn add(self, wire: &W) -> Self {
        self.add_term(wire, Coeff::One)
    }

    /// Subtracts a wire from the linear expression by adding with a coefficient of $-1$.
    fn sub(self, wire: &W) -> Self {
        self.add_term(wire, Coeff::NegativeOne)
    }
}

/// This is a trivial implementation for drivers that do not need to do anything
/// with a linear expression.
impl<W: Clone, F: Field> LinearExpression<W, F> for () {
    fn add_term(self, _: &W, _: Coeff<F>) -> Self {
        self
    }

    fn gain(self, _: Coeff<F>) -> Self {
        self
    }
}

/// A straightforward linear expression that directly computes the sum.
pub struct DirectSum<F: Field> {
    /// The current value of the linear combination.
    pub value: F,

    /// The current gain of the linear combination.
    pub current_gain: Coeff<F>,
}

impl<F: Field> Default for DirectSum<F> {
    fn default() -> Self {
        Self {
            value: F::ZERO,
            current_gain: Coeff::One,
        }
    }
}

impl<F: Field> LinearExpression<F, F> for DirectSum<F> {
    fn add_term(mut self, wire: &F, coeff: Coeff<F>) -> Self {
        match coeff * self.current_gain {
            Coeff::Zero => {}
            Coeff::One => self.value += *wire,
            Coeff::Two => self.value += wire.double(),
            Coeff::NegativeOne => self.value -= *wire,
            Coeff::Arbitrary(coeff) => self.value += *wire * coeff,
            Coeff::NegativeArbitrary(coeff) => self.value -= *wire * coeff,
        }

        self
    }

    fn gain(mut self, coeff: Coeff<F>) -> Self {
        self.current_gain = self.current_gain * coeff;
        self
    }
}

#[test]
fn test_linexp_direct() {
    use alloc::vec;
    use ragu_pasta::Fp;

    let acc = DirectSum::default()
        .add_term(&Fp::from(2), Coeff::Arbitrary(Fp::from(3))) // acc = 0 + 2 * 3 = 6
        .add_term(&Fp::from(4), Coeff::Arbitrary(Fp::from(5))) // acc = 6 + 4 * 5 = 26
        .add(&Fp::from(3)) // acc = 26 + 3 * 1 = 29
        .sub(&Fp::from(10)) // acc = 29 + 10 * -1 = 19
        .extend(vec![
            (Fp::from(3), Coeff::Arbitrary(Fp::from(4))),
            (Fp::from(10), Coeff::Arbitrary(-Fp::from(3))),
        ]); // acc = 19 + (3 * 4) + (10 * -3) = 19 + 12 - 30 = 1
    assert_eq!(acc.value, Fp::ONE);
}

#[test]
#[allow(clippy::unit_cmp)]
fn test_linexp_trivial() {
    use alloc::vec;
    use ragu_pasta::Fp;

    assert_eq!(
        (),
        ().extend(vec![
            (Fp::from(3), Coeff::Arbitrary(Fp::from(4))),
            (Fp::from(10), Coeff::Arbitrary(-Fp::from(3))),
        ])
    );
}
