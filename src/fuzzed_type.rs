/// Optionally creates a new error from a byte iterator.
///
/// When implementing this trait it is important to consider the probability of
/// creating an error or not. For example if new_err always returns an error,
/// your library will only be tested as if it always fails. The opposite is true
/// for an implementation that never returns an error.
///
/// As a general rule of thumb, this trait should be implemented to return an
/// error ~50% of the time.
///
/// # Example
///
/// Here we will implement the fuzzed error type for an enum.
///
/// ```rust
/// use embedded_hal_fuzz::error::FuzzedError;
///
/// // Required for example only.
/// use num_derive::FromPrimitive;
/// use num_traits::FromPrimitive;
///
/// #[derive(FromPrimitive)]
/// enum MyError {
///    Foo = 0,
///    Bar = 1,
/// }
///
/// impl Default for MyError {
///    fn default() -> Self {
///       MyError::Foo
///    }
/// }
///
/// impl<'a> FuzzedError<'a> for MyError {
///    fn maybe_err(data: &mut impl Iterator<Item=&'a u8>) -> Result<(), Self> {
///        if let Some(byte) = data.next().copied() {
///             match MyError::from_u8(byte) {
///                 Some(x) => Err(x),
///                 None => Err(MyError::default()),
///            }
///       } else {
///          Ok(())
///      }
///    }
/// }
/// ```

pub trait FuzzedError<'a>: Default {
    fn maybe_err(data: &mut impl Iterator<Item = &'a u8>) -> Result<(), Self>
    where
        Self: Sized;
}

/// In the case that you only care about an error/success you can use this error
/// implementation which will return a unit type error ~50% of the time.
impl<'a> FuzzedError<'a> for () {
    fn maybe_err(data: &mut impl Iterator<Item = &'a u8>) -> Result<(), Self> {
        if let Some(byte) = data.next() {
            if *byte & 0x01 == 0 {
                return Err(());
            }
        }
        Ok(())
    }
}

pub struct ConversionError;

pub trait Fuzzed<'a> {
    fn new_fuzzed(data: &mut impl Iterator<Item = &'a u8>) -> Result<Self, ConversionError>
    where
        Self: Sized;
}

impl<'a> Fuzzed<'a> for u8 {
    fn new_fuzzed(data: &mut impl Iterator<Item = &'a u8>) -> Result<Self, ConversionError> {
        data.next().copied().ok_or(ConversionError)
    }
}

fn try_get_buffer<'a, const N: usize>(
    data: &mut impl Iterator<Item = &'a u8>,
) -> Result<[u8; N], ConversionError> {
    let mut buffer = [0u8; N];
    for element in buffer.iter_mut() {
        *element = *data.next().ok_or(ConversionError)?;
    }
    Ok(buffer)
}

impl<'a> Fuzzed<'a> for u16 {
    fn new_fuzzed(data: &mut impl Iterator<Item = &'a u8>) -> Result<Self, ConversionError> {
        Ok(u16::from_le_bytes(try_get_buffer(data)?))
    }
}

impl<'a> Fuzzed<'a> for u32 {
    fn new_fuzzed(data: &mut impl Iterator<Item = &'a u8>) -> Result<Self, ConversionError> {
        Ok(u32::from_le_bytes(try_get_buffer(data)?))
    }
}

impl<'a> Fuzzed<'a> for u64 {
    fn new_fuzzed(data: &mut impl Iterator<Item = &'a u8>) -> Result<Self, ConversionError> {
        Ok(u64::from_le_bytes(try_get_buffer(data)?))
    }
}

impl<'a> Fuzzed<'a> for u128 {
    fn new_fuzzed(data: &mut impl Iterator<Item = &'a u8>) -> Result<Self, ConversionError> {
        Ok(u128::from_le_bytes(try_get_buffer(data)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unit_error() {
        let average_errors = (0..255u8)
            .map(|x| <()>::maybe_err(&mut [x.into()].iter()))
            .filter(|x| x.is_err())
            .count();
        assert_eq!(average_errors, 255 / 2 + 1);
    }
}
