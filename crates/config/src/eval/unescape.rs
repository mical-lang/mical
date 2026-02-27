use crate::Error;
use mical_cli_syntax::{TextRange, TextSize};

pub(super) fn unescape(
    text: &str,
    result: &mut String,
    base_offset: TextSize,
    errors: &mut Vec<Error>,
) {
    let mut chars = text.chars();
    let mut offset: u32 = base_offset.into();
    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        let Some(c) = chars.next() else {
            let start = offset + result.len() as u32;
            let range = TextRange::at(start.into(), 1.into());
            errors.push(Error::EmptyExpace { range });
            continue;
        };
        result.push(match c {
            '"' | '\'' | '\\' => c,
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            other => {
                let start = offset + result.len() as u32;
                let len = 1 + other.len_utf8() as u32;
                let range = TextRange::at(start.into(), len.into());
                errors.push(Error::InvalidEscape { range, sequence: format!("\\{other}") });
                other
            }
        });
        offset += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unescape2(text: &str) -> (String, Vec<Error>) {
        let mut result = String::new();
        let mut errors = Vec::new();
        unescape(text, &mut result, 0.into(), &mut errors);
        (result, errors)
    }

    #[test]
    fn basic() {
        let (result, errors) = unescape2(r#"hello \"world\""#);
        assert_eq!(result, r#"hello "world""#);
        assert!(errors.is_empty());

        let (result, errors) = unescape2(r"a\\b");
        assert_eq!(result, r"a\b");
        assert!(errors.is_empty());

        let (result, errors) = unescape2(r"line1\nline2");
        assert_eq!(result, "line1\nline2");
        assert!(errors.is_empty());

        let (result, errors) = unescape2(r"\t\r");
        assert_eq!(result, "\t\r");
        assert!(errors.is_empty());

        let (result, errors) = unescape2(r"can\'t");
        assert_eq!(result, "can't");
        assert!(errors.is_empty());
    }

    #[test]
    fn invalid() {
        let (result, errors) = unescape2(r"invalid\escape");
        assert_eq!(result, "invalidescape");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(&errors[0], Error::InvalidEscape { sequence, range } if sequence == r"\e" && *range == TextRange::at(7.into(), 2.into()))
        );
    }

    #[test]
    fn empty() {
        let (result, errors) = unescape2(r"\");
        assert_eq!(result, "");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(&errors[0], Error::EmptyExpace { range } if *range == TextRange::at(0.into(), 1.into()))
        );

        let (result, errors) = unescape2(r"empty\");
        assert_eq!(result, "empty");
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(&errors[0], Error::EmptyExpace { range } if *range == TextRange::at(5.into(), 1.into()))
        );
    }
}
