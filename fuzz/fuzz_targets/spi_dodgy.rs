#![no_main]
/// This fuzz test should pass.
use embedded_hal::spi::blocking::SpiBus;
use embedded_hal_fuzz as hal_fuzz;
use hal_fuzz::{shared_data::FuzzData, spi::{DefaultSpiError, SpiFuzz}};
use libfuzzer_sys::fuzz_target;

struct GoodDriver<T: SpiBus> {
    spi: T,
}

impl<T: SpiBus> GoodDriver<T> {
    fn new(spi: T) -> Self {
        Self { spi }
    }

    fn get_scaled_value(&mut self) -> Result<f32, ()> {
        let mut buffer = [0u8; 1];
        self.spi.transfer_in_place(&mut buffer).map_err(|_| ())?;
        if buffer[0] == 0xFF {
            panic!("This will fail the fuzz test!!")
        }
        Ok(buffer[0] as f32)
    }
}

fuzz_target!(|data: &[u8]| {
    // Ignore empty inputs.
    if data.len() > 0 {
        let data = FuzzData::new(data);
        let spi: SpiFuzz<'_, DefaultSpiError> = SpiFuzz::new(data);
        let mut driver = GoodDriver::new(spi);
        let _ = driver.get_scaled_value();
    }
});
