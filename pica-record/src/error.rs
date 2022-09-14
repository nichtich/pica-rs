use thiserror::Error;

/// An error that can occur when parsing PICA+ records.
#[derive(Error, PartialEq, Eq, Debug)]
pub enum ParsePicaError {
    #[error("invalid subfield")]
    InvalidSubfield,
}
