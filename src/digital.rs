use crate::error::FuzzedError;
use crate::shared_data::FuzzData;
use core::fmt::Debug;
use core::marker::PhantomData;
use embedded_hal::digital::{
    blocking::{InputPin, OutputPin},
    ErrorType,
};

pub trait Error<'a>: Debug + FuzzedError<'a> {}
impl<'a, E: Debug + FuzzedError<'a>> Error<'a> for E {}

/// A fuzzed backend for the digital input trait.
pub struct InputPinFuzz<'a, E: Error<'a>> {
    data: FuzzData<'a>,
    _e: PhantomData<E>,
}

impl<'a, E: Error<'a>> InputPinFuzz<'a, E> {
    pub fn new(data: FuzzData<'a>) -> Self {
        InputPinFuzz {
            data,
            _e: PhantomData,
        }
    }

    fn pin_state(&self) -> Result<bool, E> {
        // Randomly create an error.
        let mut data = match self.data.iter.try_lock() {
            Ok(data) => data,
            Err(_) => return Err(Default::default()),
        };
        E::maybe_err(&mut *data)?;
        let byte = data.next();
        match byte {
            Some(x) if *x > 128 => Ok(false),
            _ => Ok(true),
        }
    }
}

impl<'a, E: FuzzedError<'a> + Debug> ErrorType for InputPinFuzz<'a, E> {
    type Error = E;
}

impl<'a, E: Error<'a>> InputPin for InputPinFuzz<'a, E> {
    fn is_high(&self) -> Result<bool, Self::Error> {
        self.pin_state()
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        self.pin_state()
    }
}

/// A fuzzed backend for the digital input trait.
pub struct OutputPinFuzz<'a, E: FuzzedError<'a>> {
    data: FuzzData<'a>,
    _e: PhantomData<E>,
}

impl<'a, E: FuzzedError<'a>> OutputPinFuzz<'a, E> {
    pub fn new(data: FuzzData<'a>) -> Self {
        OutputPinFuzz {
            data,
            _e: PhantomData,
        }
    }

    fn maybe_err(&self) -> Result<(), E> {
        // Randomly create an error.
        let mut data = match self.data.iter.try_lock() {
            Ok(data) => data,
            Err(_) => return Err(Default::default()),
        };
        E::maybe_err(&mut *data)?;
        Ok(())
    }
}

impl<'a, E: Error<'a>> ErrorType for OutputPinFuzz<'a, E> {
    type Error = E;
}

impl<'a, E: Error<'a>> OutputPin for OutputPinFuzz<'a, E> {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.maybe_err()
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.maybe_err()
    }
}
