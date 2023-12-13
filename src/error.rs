use std::num::ParseIntError;

use num_traits::ParseFloatError;

/// Errors encountered by sms-2dm.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The card `{0}` is expected but missing")]
    MissingCard(String),
    #[error("The card `{0}` is seen more times than expected")]
    ExtraneousCard(String),
    #[error("An empty line is encountered")]
    EmptyLine,
    #[error("Expected a `{expect}` card but found a `{actual}` card")]
    WrongCardTag { expect: String, actual: String },
    #[error("{0}")]
    InvalidInt(#[from] ParseIntError),
    #[error("{0}")]
    InvalidFloat(ParseFloatError),
    #[error("A value was expected but missing")]
    MissingValue,
    #[error("An extraneous value was encountered: {0}")]
    ExtraneousValue(String),
}
impl From<ParseFloatError> for Error {
    fn from(err: ParseFloatError) -> Self {
        Self::InvalidFloat(err)
    }
}
