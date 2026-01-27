//! Conversion traits for the Universal Message Format hub model.
//!
//! These traits provide a consistent interface for converting between
//! protocol-specific types and UMF's internal format.

use thiserror::Error;

/// Errors that can occur during format conversion.
#[derive(Debug, Error)]
pub enum ConversionError {
    /// A required field is missing from the source data.
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// The data format is invalid or unexpected.
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// Serialization or deserialization failed.
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Convert a protocol-specific type to UMF internal format.
///
/// This is an infallible conversion - implementations should handle
/// missing optional fields gracefully.
pub trait ToInternal<T> {
    /// Convert self into the internal format.
    fn to_internal(self) -> T;
}

/// Convert from UMF internal format to a protocol-specific type.
///
/// This is an infallible conversion - implementations should handle
/// missing optional fields gracefully.
pub trait FromInternal<T> {
    /// Create self from the internal format.
    fn from_internal(internal: T) -> Self;
}

/// Fallible conversion to UMF internal format.
///
/// Use this when the conversion might fail due to missing required
/// fields or invalid data.
pub trait TryToInternal<T> {
    /// Try to convert self into the internal format.
    fn try_to_internal(self) -> Result<T, ConversionError>;
}

/// Fallible conversion from UMF internal format.
///
/// Use this when the conversion might fail due to protocol-specific
/// requirements that the internal format doesn't guarantee.
pub trait TryFromInternal<T>: Sized {
    /// Try to create self from the internal format.
    fn try_from_internal(internal: T) -> Result<Self, ConversionError>;
}
