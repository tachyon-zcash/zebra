use ragu_core::{Result, drivers::Driver, maybe::Maybe};

use core::marker::PhantomData;

use crate::{Element, io::Buffer};

/// Implementation of [`Buffer`] that acts as a pipe, writing elements to an
/// underlying buffer through allocation.
pub struct Pipe<'a, 'dr, D: Driver<'dr>, B: Buffer<'dr, D>> {
    dr: &'a mut D,
    buf: B,
    _marker: PhantomData<&'dr ()>,
}

impl<'a, 'dr, D: Driver<'dr>, B: Buffer<'dr, D>> Pipe<'a, 'dr, D, B> {
    /// Creates a new pipe, given a destination driver and buffer for that
    /// driver. [`Element`]s written to this pipe will be allocated on the
    /// destination driver with the values taken from the source driver
    /// [`Element`]s.
    pub fn new(dr: &'a mut D, buf: B) -> Self {
        Pipe {
            dr,
            buf,
            _marker: PhantomData,
        }
    }
}

impl<'dr, S: Driver<'dr, F = D::F>, D: Driver<'dr>, B: Buffer<'dr, D>> Buffer<'dr, S>
    for Pipe<'_, 'dr, D, B>
{
    fn write(&mut self, _: &mut S, value: &Element<'dr, S>) -> Result<()> {
        let elem = Element::alloc(self.dr, D::just(|| *value.value().take()))?;
        self.buf.write(self.dr, &elem)
    }
}

#[test]
fn test_pipe_between_wireless_emulators() -> Result<()> {
    use alloc::vec::Vec;
    use ragu_core::drivers::emulator::{Emulator, Wireless};
    use ragu_core::maybe::{Always, MaybeKind};
    use ragu_pasta::Fp;

    // Create first wireless emulator and allocate some elements
    let mut source_dr: Emulator<Wireless<Always<()>, Fp>> = Emulator::execute();
    let values = [Fp::from(42u64), Fp::from(123u64), Fp::from(999u64)];
    let source_elements: Vec<Element<'_, _>> = values
        .iter()
        .map(|&v| Element::alloc(&mut source_dr, Always::maybe_just(|| v)))
        .collect::<Result<_>>()?;

    // Create second wireless emulator and use pipe to transfer elements
    let mut dest_dr: Emulator<Wireless<Always<()>, Fp>> = Emulator::execute();
    let mut dest_buffer: Vec<Element<'_, _>> = Vec::new();

    {
        let mut pipe = Pipe::new(&mut dest_dr, &mut dest_buffer);
        for elem in &source_elements {
            pipe.write(&mut source_dr, elem)?;
        }
    }

    // Verify destination buffer has the same values
    assert_eq!(dest_buffer.len(), values.len());
    for (dest_elem, &expected_value) in dest_buffer.iter().zip(values.iter()) {
        assert_eq!(*dest_elem.value().take(), expected_value);
    }

    Ok(())
}
