use crate::error::FuzzedError;
use crate::fuzzed_type::Fuzzed;
use crate::shared_data::FuzzData;
use core::{fmt::Debug, marker::PhantomData};
use embedded_hal::spi::{
    self,
    blocking::{SpiBus, SpiBusFlush, SpiBusRead, SpiBusWrite},
    ErrorKind, ErrorType,
};

pub trait Word<'a>: Copy + Fuzzed<'a> {}
impl<'a, T: Copy + Fuzzed<'a>> Word<'a> for T {}

#[derive(Debug)]
pub struct DefaultSpiError(ErrorKind);

impl spi::Error for DefaultSpiError {
    fn kind(&self) -> ErrorKind {
        self.0
    }
}

impl Default for DefaultSpiError {
    fn default() -> Self {
        DefaultSpiError(ErrorKind::Other)
    }
}

impl<'a> FuzzedError<'a> for DefaultSpiError {
    fn maybe_err(data: &mut impl Iterator<Item = &'a u8>) -> Result<(), Self> {
        match data.next().copied().map(|x| x % 6) {
            Some(0) => Err(DefaultSpiError(ErrorKind::Overrun)),
            Some(1) => Err(DefaultSpiError(ErrorKind::ModeFault)),
            Some(2) => Err(DefaultSpiError(ErrorKind::FrameFormat)),
            Some(3) => Err(DefaultSpiError(ErrorKind::ChipSelectFault)),
            Some(4) => Err(DefaultSpiError(ErrorKind::Other)),
            _ => Ok(()),
        }
    }
}

pub trait Error<'a>: FuzzedError<'a> + Debug + embedded_hal::spi::Error {}
impl<'a, E: FuzzedError<'a> + Debug + embedded_hal::spi::Error> Error<'a> for E {}

impl<'a, E: Error<'a>, W: Word<'a>> ErrorType for SpiFuzz<'a, E, W> {
    type Error = E;
}

pub struct SpiFuzz<'a, E = DefaultSpiError, W: Word<'a> = u8> {
    data: FuzzData<'a>,
    e_: PhantomData<E>,
    w_: PhantomData<W>,
}

impl<'a, E: Error<'a>, W: Word<'a>> SpiFuzz<'a, E, W> {
    pub fn new(data: FuzzData<'a>) -> Self {
        SpiFuzz {
            data,
            e_: PhantomData,
            w_: PhantomData,
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

impl<'a, E: Error<'a>, W: Word<'a>> SpiBusFlush for SpiFuzz<'a, E, W> {
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.maybe_err()
    }
}

impl<'a, E: Error<'a>, W: Word<'a>> SpiBusWrite<W> for SpiFuzz<'a, E, W> {
    fn write(&mut self, _words: &[W]) -> Result<(), Self::Error> {
        self.maybe_err()
    }
}

impl<'a, E: Error<'a>, W: Word<'a>> SpiBusRead<W> for SpiFuzz<'a, E, W> {
    fn read(&mut self, buffer: &mut [W]) -> Result<(), Self::Error> {
        // Randomly create an error.
        let mut data = match self.data.iter.try_lock() {
            Ok(data) => data,
            Err(_) => return Err(Default::default()),
        };
        E::maybe_err(&mut *data)?;
        for element in buffer.iter_mut() {
            *element = W::new_fuzzed(&mut *data).map_err(|_| Default::default())?;
        }
        Ok(())
    }
}

impl<'a, E: Error<'a>, W: Word<'a>> SpiBus<W> for SpiFuzz<'a, E, W> {
    fn transfer(&mut self, read: &mut [W], write: &[W]) -> Result<(), Self::Error> {
        self.write(write)?;
        self.read(read)
    }
    fn transfer_in_place(&mut self, words: &mut [W]) -> Result<(), Self::Error> {
        self.read(words)
    }
}
