#![no_main]
/// This fuzz test should pass.
use embedded_hal::{digital::blocking::InputPin, spi::blocking::SpiBus};
use embedded_hal_fuzz as hal_fuzz;
use hal_fuzz::{
    digital::InputPinFuzz,
    shared_data::FuzzData,
    spi::{DefaultSpiError, SpiFuzz},
};
use libfuzzer_sys::fuzz_target;

struct Driver<T: SpiBus, P: InputPin> {
    spi: T,
    pin: P,
}

impl<T: SpiBus, P: InputPin> Driver<T, P> {
    fn new(spi: T, pin: P) -> Self {
        Self { spi, pin }
    }

    fn get_scaled_value(&mut self) -> Result<u16, ()> {
        if !self.pin.is_high().map_err(|_| ())? {
            return Ok(0);
        }
        let mut buffer = [0u8; 2];
        self.spi.transfer_in_place(&mut buffer).map_err(|_| ())?;
        Ok(u16::from_le_bytes(buffer))
    }
}

type Error = ();

fuzz_target!(|data: &[u8]| {
    // Ignore empty inputs.
    if data.len() > 0 {
        let data = FuzzData::new(data);
        let spi: SpiFuzz<'_, DefaultSpiError> = SpiFuzz::new(data.clone());
        let pin: InputPinFuzz<'_, Error> = InputPinFuzz::new(data.clone());
        let mut driver = Driver::new(spi, pin);
        let _ = driver.get_scaled_value();
    }
});
