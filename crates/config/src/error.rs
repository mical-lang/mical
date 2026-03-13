use core::fmt;
use mical_cli_syntax::TextRange;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidEscape { range: TextRange, sequence: String },
    EmptyEscape { range: TextRange },
    InvalidRadixDigits { range: TextRange, text: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidEscape { range, sequence } => {
                write!(f, "invalid escape sequence '{}' at {:?}", sequence, range)
            }
            Error::EmptyEscape { range } => {
                write!(f, "empty escape at {:?}", range)
            }
            Error::InvalidRadixDigits { range, text } => {
                write!(f, "invalid digits for radix in '{}' at {:?}", text, range)
            }
        }
    }
}
