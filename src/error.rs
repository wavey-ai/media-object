use std::fmt;

/// Errors returned while constructing, validating, encoding, or decoding an object.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// An operational limit exceeds the crate's hard safety ceiling.
    InvalidLimit {
        field: &'static str,
        actual: usize,
        hard_maximum: usize,
    },
    /// Input exceeds a configured bound.
    LimitExceeded {
        field: &'static str,
        actual: usize,
        maximum: usize,
    },
    /// A required string is empty.
    EmptyField(&'static str),
    /// A field violates a model invariant.
    InvalidField {
        field: &'static str,
        reason: &'static str,
    },
    /// The payload bytes do not match the hash carried by the object key.
    PayloadHashMismatch,
    /// A dependency points back to the object that contains it.
    SelfDependency,
    /// A set-like field contains a duplicate value.
    DuplicateValue(&'static str),
    /// Set-like values on the wire are not in their canonical order.
    NonCanonicalOrder(&'static str),
    /// The envelope magic does not identify a media-object frame.
    InvalidMagic,
    /// The envelope uses a wire version this crate does not implement.
    UnsupportedVersion(u16),
    /// A reserved field is non-zero.
    ReservedField(&'static str),
    /// A numeric tag is outside the defined enum variants.
    UnknownTag { field: &'static str, value: u8 },
    /// A byte intended to encode a boolean is neither zero nor one.
    InvalidBoolean { field: &'static str, value: u8 },
    /// A length-delimited string is not valid UTF-8.
    InvalidUtf8(&'static str),
    /// The input ends before a complete field can be read.
    Truncated,
    /// The outer frame length and actual body length differ.
    DeclaredLengthMismatch { declared: usize, actual: usize },
    /// Bytes remain after all v1 fields have been decoded.
    TrailingBytes(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLimit {
                field,
                actual,
                hard_maximum,
            } => write!(
                formatter,
                "limit {field} is {actual}, above the hard maximum {hard_maximum}"
            ),
            Self::LimitExceeded {
                field,
                actual,
                maximum,
            } => write!(
                formatter,
                "{field} is {actual} bytes/items, above the maximum {maximum}"
            ),
            Self::EmptyField(field) => write!(formatter, "{field} must not be empty"),
            Self::InvalidField { field, reason } => {
                write!(formatter, "invalid {field}: {reason}")
            }
            Self::PayloadHashMismatch => {
                formatter.write_str("payload hash does not match object key")
            }
            Self::SelfDependency => formatter.write_str("an object cannot depend on itself"),
            Self::DuplicateValue(field) => write!(formatter, "{field} contains a duplicate value"),
            Self::NonCanonicalOrder(field) => {
                write!(formatter, "{field} is not in canonical order")
            }
            Self::InvalidMagic => formatter.write_str("invalid media-object envelope magic"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported media-object wire version {version}")
            }
            Self::ReservedField(field) => write!(formatter, "reserved field {field} is non-zero"),
            Self::UnknownTag { field, value } => {
                write!(formatter, "unknown {field} tag {value}")
            }
            Self::InvalidBoolean { field, value } => {
                write!(formatter, "invalid {field} boolean byte {value}")
            }
            Self::InvalidUtf8(field) => write!(formatter, "{field} is not valid UTF-8"),
            Self::Truncated => formatter.write_str("truncated media-object envelope"),
            Self::DeclaredLengthMismatch { declared, actual } => write!(
                formatter,
                "declared body length {declared} differs from actual body length {actual}"
            ),
            Self::TrailingBytes(count) => {
                write!(formatter, "{count} trailing bytes in v1 envelope")
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;
