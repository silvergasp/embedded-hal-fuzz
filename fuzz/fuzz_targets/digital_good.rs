#![no_main]
/// This fuzz test should pass.
use embedded_hal::digital::blocking::InputPin;
use embedded_hal_fuzz as hal_fuzz;
use libfuzzer_sys::fuzz_target;

struct Driver<T: InputPin> {
    pin: T,
}

impl<T: InputPin> Driver<T> {
    fn new(pin: T) -> Self {
        Self { pin }
    }

    fn get_pinstate(&mut self) -> Result<bool, ()> {
        // If this pin is not readable it will panic. This would fail a fuzz
        // test.
        match self.pin.is_high() {
            Ok(a) => Ok(a),
            Err(_) => Err(()),
        }
    }
}

type Error = ();

fuzz_target!(|data: &[u8]| {
    use hal_fuzz::shared_data::FuzzData;
    // Ignore empty inputs.
    if data.len() > 0 {
        let data = FuzzData::new(data);
        let pin: hal_fuzz::digital::InputPinFuzz<'_, Error> =
            hal_fuzz::digital::InputPinFuzz::new(data);
        let mut driver = Driver::new(pin);
        let _ = driver.get_pinstate();
    }
});
