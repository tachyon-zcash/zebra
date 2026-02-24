use super::{Coeff, Driver, DriverTypes, Field, FromDriver, Result};

/// This is a dummy driver that does absolutely nothing.
impl<F: Field> Driver<'_> for core::marker::PhantomData<F> {
    type F = F;
    type Wire = ();
    const ONE: Self::Wire = ();

    fn alloc(&mut self, _: impl Fn() -> Result<Coeff<Self::F>>) -> Result<Self::Wire> {
        Ok(())
    }

    fn mul(
        &mut self,
        _: impl Fn() -> Result<(Coeff<F>, Coeff<F>, Coeff<F>)>,
    ) -> Result<(Self::Wire, Self::Wire, Self::Wire)> {
        Ok(((), (), ()))
    }

    fn add(&mut self, _: impl Fn(Self::LCadd) -> Self::LCadd) -> Self::Wire {}

    fn enforce_zero(&mut self, _: impl Fn(Self::LCenforce) -> Self::LCenforce) -> Result<()> {
        Ok(())
    }
}

impl<F: Field> DriverTypes for core::marker::PhantomData<F> {
    type ImplField = F;
    type ImplWire = ();
    type MaybeKind = crate::maybe::Empty;
    type LCadd = ();
    type LCenforce = ();
}

impl<'dr, 'new_dr, D: Driver<'dr>> FromDriver<'dr, 'new_dr, D> for core::marker::PhantomData<D::F> {
    type NewDriver = Self;

    fn convert_wire(&mut self, _: &D::Wire) -> Result<<Self::NewDriver as Driver<'new_dr>>::Wire> {
        Ok(())
    }
}
