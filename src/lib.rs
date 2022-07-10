//! This crate is specifically designed for fuzzing device drivers or full apps.
//! It provides a best guess for how to fuzz device-drivers/apps.
//!
//! # Getting started
//! If you are not familiar with fuzzing in rust then it is recommended that you
//! read over the [cargo-fuzz book](https://rust-fuzz.github.io/book/cargo-fuzz.html).
//!
//! ## Install cargo-fuzz
//! To install cargo-fuzz, run:
//! ```bash
//! cargo install cargo-fuzz
//! ```
//!
//! ## Initialising cargo fuzz in your project
//! To set your project up to use cargo fuzz, run:
//! ```bash
//! cargo-fuzz init
//! ```
//! This will add a set to targets under the 'fuzz' directory.
//!
//! ## Adding a new embedded fuzz target
//! To add a new embedded fuzz target, run:
//! ```bash
//! cargo-fuzz add <my_target>
//! ```
//! This will add a new binary target 'fuzz/fuzz_targets/my_target.rs'.
//! by default this your new target will look something like this;
//! ```no_run
//! #![no_main]
//! use libfuzzer_sys::fuzz_target;
//!
//! fuzz_target!(|data: &[u8]| {
//!     // fuzzed code goes here
//! });
//! ```
//!
//! Let's create a small driver with an intentional bug. We will create a fuzz
//! target for this driver to see if it will fail.
//! ```
//! use embedded_hal::i2c::blocking::I2c;
//! struct DodgyDriver<T: I2c> {
//!     i2c: T,
//! }
//!
//! impl<T: I2c> DodgyDriver<T> {
//!     fn new(i2c: T) -> Self {
//!         Self { i2c }
//!     }
//!
//!     fn get_dodgy_scaled_value(&mut self) -> Result<u8, ()> {
//!         let mut buffer = [0u8; 1];
//!         self.i2c.read(0x01, &mut buffer).map_err(|_| ())?;
//!         let a = buffer[0];
//!         self.i2c
//!             .write_read(0x01, &[1, 2, 3, 4], &mut buffer)
//!             .map_err(|_| ())?;
//!         // BUG: May overflow the u8 integer.
//!         Ok(buffer[0] * a)
//!     }
//! }
//! ```
//!
//! Lets go ahead and modify our fuzz target to test this driver.
//! ```no_run
//! #![no_main]
//! # use embedded_hal::i2c::blocking::I2c;
//! # struct DodgyDriver<T: I2c> {
//! #     i2c: T,
//! # }
//! # impl<T: I2c> DodgyDriver<T> {
//! #     fn new(i2c: T) -> Self {
//! #         Self { i2c }
//! #     }
//! #     fn get_dodgy_scaled_value(&mut self) -> Result<u8, ()> {
//! #         let mut buffer = [0u8; 1];
//! #         self.i2c.read(0x01, &mut buffer).map_err(|_| ())?;
//! #         let a = buffer[0];
//! #         self.i2c
//! #             .write_read(0x01, &[1, 2, 3, 4], &mut buffer)
//! #             .map_err(|_| ())?;
//! #         // BUG: May overflow the u8 integer.
//! #         Ok(buffer[0] * a)
//! #     }
//! # }
//! use embedded_hal_fuzz as hal_fuzz;
//! use libfuzzer_sys::fuzz_target;
//!
//!
//! fuzz_target!(|data: &[u8]| {
//!     // Ignore empty inputs.
//!     if data.len() > 0 {
//!         use hal_fuzz::shared_data::FuzzData;
//!         let data = FuzzData::new(data);
//!         let i2c: hal_fuzz::i2c::I2cFuzz = hal_fuzz::i2c::I2cFuzz::new(data);
//!         let mut driver = DodgyDriver::new(i2c);
//!         // NOTE: we don't care about getting an error only about a crash, so
//!         // we discard the result.
//!         let _ = driver.get_dodgy_scaled_value();
//!     }
//! });
//! ```
//!
//! Finally we can run the fuzz test with cargo-fuzz.
//! ```bash
//! cargo fuzz run my_target
//! ```
//!
//! ## Fuzzing complete apps
//! To fuzz a complete app you will need to structure your app similar to how
//! you would structure a driver. e.g.
//! ```rust, no_run
//! #![no_main]
//! use embedded_hal::blocking::i2c::WriteRead;
//! struct MyApp<T: WriteRead> {
//!     i2c: T,
//! }
//! impl<T: WriteRead> MyApp<T> {
//!     fn new(i2c: T) -> Self {
//!         Self { i2c }
//!     }
//!     // ...
//! }
//! use embedded_hal_fuzz as hal_fuzz;
//! use libfuzzer_sys::fuzz_target;
//!
//!
//! fuzz_target!(|data: &[u8]| {
//!     // Ignore empty inputs.
//!     if data.len() > 0 {
//!         use hal_fuzz::shared_data::FuzzData;
//!         let data = FuzzData::new(data);
//!         let i2c: hal_fuzz::i2c::I2cFuzz<'_, ()> = hal_fuzz::i2c::I2cFuzz::new(data);
//!         let mut driver = MyApp::new(i2c);
//!         // ...
//!     }
//! });
//! ```
//!
//! ## Custom errors
//! You might have noticed that in the previous example we are using the unit type
//! `()` as out error type. This is conveniant for fuzzing when we only care about
//! if we get an error or not. However sometimes we want to have a custom error
//! type. Luckily this is supported by the embedded-hal-fuzzer crate. Checkout the
//! [error](crate::error) module for more information.
//!
//! ## Examples
//! For some examples of how to use the fuzzer, checkout the targets in the
//! 'fuzz/fuzz_targets' directory of this repository.

pub mod delay;
pub mod digital;
pub mod error;
pub mod fuzzed_type;
pub mod i2c;
pub mod serial;
pub mod shared_data;
pub mod spi;
