#[cfg(feature = "std")]
use std::fmt;

/// Error type when deserialization fails.
///
/// Miniserde errors contain no information about what went wrong. **If you need
/// more than no information, use Serde.**
#[derive(Copy, Clone, Debug)]
pub struct Error;

/// Result type returned by deserialization functions.
pub type Result<T> = crate::lib::result::Result<T, Error>;

#[cfg(feature = "std")]
impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("miniserde error")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn description(&self) -> &str {
        "miniserde error"
    }
}
