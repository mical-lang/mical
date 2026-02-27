use core::fmt;
use mical_cli_syntax::TextRange;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidEscape { range: TextRange, sequence: String },
    EmptyExpace { range: TextRange },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidEscape { range, sequence } => {
                write!(f, "invalid escape sequence '{}' at {:?}", sequence, range)
            }
            Error::EmptyExpace { range } => {
                write!(f, "empty expace at {:?}", range)
            }
        }
    }
}
