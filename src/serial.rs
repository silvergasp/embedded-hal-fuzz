use crate::error::FuzzedError;
use crate::shared_data::FuzzData;
use core::marker::PhantomData;
use embedded_hal::serial::{
    blocking::Write as BlockingWrite, nb::Read as NbRead, nb::Write as NbWrite, ErrorType,
};

pub trait Error<'a>: FuzzedError<'a> + embedded_hal::serial::Error {}
impl<'a, E: FuzzedError<'a> + embedded_hal::serial::Error> Error<'a> for E {}

pub struct SerialFuzz<'a, E> {
    data: FuzzData<'a>,
    e_: PhantomData<E>,
}

impl<'a, E: Error<'a>> SerialFuzz<'a, E> {
    pub fn new(data: FuzzData<'a>) -> Self {
        SerialFuzz {
            data,
            e_: PhantomData,
        }
    }

    fn maybe_err(&mut self) -> Result<(), E> {
        match self.data.iter.try_lock() {
            Ok(mut data) => {
                E::maybe_err(&mut *data)?;
                Ok(())
            }
            Err(_) => Ok(()),
        }
    }
}

impl<'a, E: Error<'a>> ErrorType for SerialFuzz<'a, E> {
    type Error = E;
}

impl<'a, E: Error<'a>> BlockingWrite<u8> for SerialFuzz<'a, E> {
    fn write(&mut self, _: &[u8]) -> Result<(), Self::Error> {
        self.maybe_err()
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.maybe_err()
    }
}

impl<'a, E: Error<'a>> NbRead<u8> for SerialFuzz<'a, E> {
    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let mut data = match self.data.iter.try_lock() {
            Ok(data) => data,
            Err(_) => return Err(nb::Error::WouldBlock),
        };
        E::maybe_err(&mut *data).map_err(nb::Error::Other)?;
        data.next()
            .copied()
            .ok_or_else(|| nb::Error::Other(E::default()))
    }
}

impl<'a, E: Error<'a>> NbWrite<u8> for SerialFuzz<'a, E> {
    fn write(&mut self, _word: u8) -> nb::Result<(), Self::Error> {
        let mut data = match self.data.iter.lock() {
            Ok(data) => data,
            Err(_) => return Err(nb::Error::WouldBlock),
        };
        // Create an error/or not based on fuzzed data.
        E::maybe_err(&mut *data).map_err(nb::Error::Other)?;
        Ok(())
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        // Write is a no-op an will sometimes return an error.
        // We are looking for the same functionality with flush.
        <Self as NbWrite>::write(self, 0)
    }
}
