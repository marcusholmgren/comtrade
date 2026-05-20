use std::num::NonZeroUsize;
use thiserror::Error;

/// The primary error type for all COMTRADE parsing and validation operations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ComtradeError {
    /// Raised when a line in the .cfg file has fewer comma-separated elements than required.
    #[error("Missing elements on line. Context: {0}")]
    MissingLineElements(&'static str),

    /// Raised when a specific field cannot be parsed into its expected type.
    #[error("Unable to parse value: {value} as {type_} for {field}.")]
    InvalidValue {
        /// The raw string value that failed to parse.
        value: String,
        /// The name of the target type.
        type_: &'static str,
        /// The context or field name.
        field: &'static str,
    },

    /// Raised when the .cfg file ends abruptly before all expected sections are read.
    #[error("Unexpected end of cfg file.")]
    UnexpectedEndOfCfgFile,

    /// Raised when the COMTRADE revision string is not recognized (must be 1991, 1999, or 2013).
    #[error("Invalid version string: {0}")]
    BadRevisionFormat(String),

    /// Raised when the digital normal status value is invalid (must be 0 or 1).
    #[error("The normal status for status channel index {0} is invalid. It must be 0 or 1.")]
    InvalidNormalStatus(NonZeroUsize),

    /// Raised for general parsing or format errors with custom messages.
    #[error("Parser Error: {0}")]
    ParserError(String),

    /// Raised when the timestamp precision cannot be determined from the fractional part.
    #[error("Unable to find timestamp precision")]
    CantFindTimestampPrecision,
}

impl ComtradeError {
    /// Adds or updates context information for the error, if applicable.
    pub fn add_context(self, msg: &'static str) -> Self {
        match self {
            ComtradeError::MissingLineElements(_) => ComtradeError::MissingLineElements(msg),
            ComtradeError::InvalidValue {
                value,
                type_,
                field: _,
            } => ComtradeError::InvalidValue {
                value,
                type_,
                field: msg,
            },
            _ => self,
        }
    }
}
