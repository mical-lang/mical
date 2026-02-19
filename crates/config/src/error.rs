use core::fmt;
use mical_cli_syntax::TextRange;

/// Error encountered during AST-to-Config evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// A required syntax element (key, value, token, etc.) was missing or
    /// malformed in the AST.  The parser should have already reported the
    /// corresponding syntax error, so this variant does not distinguish
    /// between different kinds of missing nodes.
    MissingSyntax { range: TextRange },

    /// An invalid escape sequence was found in a quoted string or quoted key.
    InvalidEscape { range: TextRange, sequence: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingSyntax { range } => {
                write!(f, "missing syntax element at {:?}", range)
            }
            ConfigError::InvalidEscape { range, sequence } => {
                write!(f, "invalid escape sequence '{}' at {:?}", sequence, range)
            }
        }
    }
}
