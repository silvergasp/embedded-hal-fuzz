use crate::{error::FuzzedError, shared_data::FuzzData};
use core::{fmt::Debug, marker::PhantomData};
use embedded_hal::delay::blocking::DelayUs;

/// A fuzzed backend for the I2C read trait.
pub struct DelayFuzz<'a, E: FuzzedError<'a> = ()> {
    data: FuzzData<'a>,
    _e: PhantomData<E>,
}

impl<'a, E: FuzzedError<'a> + Debug> DelayFuzz<'a, E> {
    pub fn new(data: FuzzData<'a>) -> Self {
        DelayFuzz {
            data,
            _e: PhantomData,
        }
    }
}
impl<'a, E: FuzzedError<'a> + Debug> DelayUs for DelayFuzz<'a, E> {
    type Error = E;

    fn delay_us(&mut self, _us: u32) -> Result<(), Self::Error> {
        // Ignore delays as that would slow fuzzing down. Technically this
        // is an output anyway, and we only care about inputs for fuzzing.

        // Randomly create an error.
        let mut data = match self.data.iter.try_lock() {
            Ok(data) => data,
            Err(_) => return Err(Default::default()),
        };
        E::maybe_err(&mut *data)?;
        Ok(())
    }
}
