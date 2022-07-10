use crate::{
    fuzzed_type::{ConversionError, Fuzzed, FuzzedError},
    shared_data::FuzzData,
};
use core::{fmt::Debug, marker::PhantomData};
use embedded_hal::i2c::{
    blocking::{I2c, Operation},
    AddressMode, Error as I2cError, ErrorKind, ErrorType, NoAcknowledgeSource,
};

pub trait Error<'a>: FuzzedError<'a> + Debug + embedded_hal::i2c::Error {}
impl<'a, E: FuzzedError<'a> + Debug + embedded_hal::i2c::Error> Error<'a> for E {}

#[derive(Debug)]
pub struct DefaultI2cError {
    kind: ErrorKind,
}

impl Default for DefaultI2cError {
    fn default() -> Self {
        DefaultI2cError {
            kind: ErrorKind::Other,
        }
    }
}

impl<'a> FuzzedError<'a> for DefaultI2cError {
    fn maybe_err(data: &mut impl Iterator<Item = &'a u8>) -> Result<(), Self> {
        match ErrorKind::new_fuzzed(&mut *data) {
            Ok(x) => Err(DefaultI2cError { kind: x }),
            Err(_) => Ok(()),
        }
    }
}

impl I2cError for DefaultI2cError {
    fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl<'a> Fuzzed<'a> for NoAcknowledgeSource {
    fn new_fuzzed(data: &mut impl Iterator<Item = &'a u8>) -> Result<Self, ConversionError> {
        match data.next().copied().map(|x| x % 3) {
            Some(0) => Ok(NoAcknowledgeSource::Address),
            Some(1) => Ok(NoAcknowledgeSource::Unknown),
            Some(2) => Ok(NoAcknowledgeSource::Unknown),
            _ => Err(ConversionError),
        }
    }
}

impl<'a> Fuzzed<'a> for ErrorKind {
    fn new_fuzzed(data: &mut impl Iterator<Item = &'a u8>) -> Result<Self, ConversionError> {
        match data.next().copied().map(|x| x % 6) {
            Some(0) => Ok(ErrorKind::Bus),
            Some(1) => Ok(ErrorKind::ArbitrationLoss),
            Some(2) => Ok(ErrorKind::NoAcknowledge(NoAcknowledgeSource::new_fuzzed(
                data,
            )?)),
            Some(3) => Ok(ErrorKind::Overrun),
            Some(4) => Ok(ErrorKind::Other),
            _ => Err(ConversionError),
        }
    }
}

/// A fuzzed backend for the I2C read trait.
pub struct I2cFuzz<'a, E: Error<'a> = DefaultI2cError> {
    data: FuzzData<'a>,
    _e: PhantomData<E>,
}

impl<'a, E: Error<'a>> I2cFuzz<'a, E> {
    pub fn new(data: FuzzData<'a>) -> Self {
        I2cFuzz {
            data,
            _e: PhantomData,
        }
    }
}

impl<'a, E: Error<'a>> ErrorType for I2cFuzz<'a, E> {
    type Error = E;
}

impl<'a, A: AddressMode + Clone, E: Error<'a>> I2c<A> for I2cFuzz<'a, E> {
    fn read(&mut self, _address: A, buffer: &mut [u8]) -> Result<(), Self::Error> {
        // Randomly create an error.
        let mut data = match self.data.iter.try_lock() {
            Ok(data) => data,
            Err(_) => return Err(Default::default()),
        };
        E::maybe_err(&mut *data)?;
        for element in buffer.iter_mut() {
            *element = *data.next().ok_or_else(Default::default)?;
        }
        Ok(())
    }
    fn write(&mut self, _address: A, _bytes: &[u8]) -> Result<(), Self::Error> {
        let mut data = match self.data.iter.try_lock() {
            Ok(data) => data,
            Err(_) => return Err(Default::default()),
        };
        E::maybe_err(&mut *data)?;
        Ok(())
    }
    fn write_iter<B>(&mut self, address: A, _bytes: B) -> Result<(), Self::Error>
    where
        B: IntoIterator<Item = u8>,
    {
        // Write data is ignored as it is an output.
        self.write(address, &[])
    }
    fn write_read(
        &mut self,
        address: A,
        _bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        // Write data is ignored as it is an output.
        self.read(address, buffer)
    }
    fn write_iter_read<B>(
        &mut self,
        address: A,
        _bytes: B,
        buffer: &mut [u8],
    ) -> Result<(), Self::Error>
    where
        B: IntoIterator<Item = u8>,
    {
        self.write_read(address, &[], buffer)
    }
    fn transaction<'b>(
        &mut self,
        address: A,
        operations: &mut [Operation<'b>],
    ) -> Result<(), Self::Error> {
        for op in operations.iter_mut() {
            match op {
                Operation::Write(bytes) => self.write(address.clone(), bytes)?,
                Operation::Read(buffer) => self.read(address.clone(), buffer)?,
            }
        }
        Ok(())
    }
    fn transaction_iter<'b, O>(&mut self, address: A, operations: O) -> Result<(), Self::Error>
    where
        O: IntoIterator<Item = Operation<'b>>,
    {
        for op in operations.into_iter() {
            match op {
                Operation::Write(bytes) => self.write(address.clone(), bytes)?,
                Operation::Read(buffer) => self.read(address.clone(), buffer)?,
            }
        }
        Ok(())
    }
}
