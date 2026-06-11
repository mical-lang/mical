use crate::lines::Line;

#[derive(Debug)]
pub struct LineScanner<'s> {
    rest: &'s str,
    terminator_len: u32,
}

impl<'s> LineScanner<'s> {
    pub(crate) fn new(line: Line<'s>) -> Self {
        LineScanner { rest: line.text, terminator_len: line.terminator_len }
    }

    pub fn empty() -> Self {
        LineScanner { rest: "", terminator_len: 0 }
    }

    pub fn rest_len(&self) -> u32 {
        self.rest.len() as u32
    }

    pub fn advance(&mut self, len: u32) {
        assert!(len as usize <= self.rest.len(), "advance past the end of the line");
        self.rest = &self.rest[len as usize..];
    }

    pub fn take_terminator(self) -> u32 {
        assert!(self.rest.is_empty(), "line text is not fully consumed");
        self.terminator_len
    }

    pub fn scan_word(&self) -> u32 {
        self.rest.bytes().take_while(|b| !matches!(b, b' ' | b'\t')).count() as u32
    }

    pub fn scan_quoted(&self) -> Option<QuotedToken> {
        let bytes = self.rest.as_bytes();
        let (quote, quote_byte) = match bytes.first()? {
            b'\'' => (Quote::Single, b'\''),
            b'"' => (Quote::Double, b'"'),
            _ => return None,
        };
        let mut i = 1;
        while i < bytes.len() {
            match bytes[i] {
                b'\\' if matches!(bytes.get(i + 1), Some(&b) if b == b'\\' || b == quote_byte) => {
                    i += 2;
                }
                b if b == quote_byte => {
                    return Some(QuotedToken { quote, content_len: (i - 1) as u32, closed: true });
                }
                _ => i += 1,
            }
        }
        Some(QuotedToken { quote, content_len: (bytes.len() - 1) as u32, closed: false })
    }

    pub fn scan_separator(&self) -> Separator {
        let bytes = self.rest.as_bytes();
        let mut space_end = 0;
        while space_end < bytes.len() && bytes[space_end] == b' ' {
            space_end += 1;
        }
        let mut tab_run_end = space_end;
        if bytes.get(tab_run_end) == Some(&b'\t') {
            while tab_run_end < bytes.len() && matches!(bytes[tab_run_end], b' ' | b'\t') {
                tab_run_end += 1;
            }
        }
        Separator { space_len: space_end as u32, tab_run_len: (tab_run_end - space_end) as u32 }
    }

    pub fn scan_whitespace_run(&self) -> Option<WhitespaceRun> {
        let bytes = self.rest.as_bytes();
        let &first = bytes.first().filter(|b| matches!(b, b' ' | b'\t'))?;
        let len = bytes.iter().take_while(|&&b| b == first).count() as u32;
        Some(WhitespaceRun { is_tab: first == b'\t', len })
    }

    pub fn split_comment(&self) -> SplitComment {
        let bytes = self.rest.as_bytes();
        let hash = (0..bytes.len()).find(|&i| bytes[i] == b'#' && (i == 0 || bytes[i - 1] == b' '));
        match hash {
            Some(hash) => {
                // The spaces immediately preceding the `#` belong to the comment:
                // the spec removes them from the line together with the comment.
                let mut comment_start = hash;
                while comment_start > 0 && bytes[comment_start - 1] == b' ' {
                    comment_start -= 1;
                }
                SplitComment {
                    value_len: comment_start as u32,
                    trailing_space_len: 0,
                    comment_len: (bytes.len() - comment_start) as u32,
                }
            }
            None => {
                let mut value_end = bytes.len();
                while value_end > 0 && bytes[value_end - 1] == b' ' {
                    value_end -= 1;
                }
                SplitComment {
                    value_len: value_end as u32,
                    trailing_space_len: (bytes.len() - value_end) as u32,
                    comment_len: 0,
                }
            }
        }
    }

    pub fn classify_value(&self, value_len: u32) -> ValueKind {
        classify(&self.rest[..value_len as usize])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quote {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuotedToken {
    pub quote: Quote,
    pub content_len: u32,
    pub closed: bool,
}

impl QuotedToken {
    pub fn consumed_len(&self) -> u32 {
        1 + self.content_len + self.closed as u32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Separator {
    pub space_len: u32,
    pub tab_run_len: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhitespaceRun {
    pub is_tab: bool,
    pub len: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitComment {
    pub value_len: u32,
    pub trailing_space_len: u32,
    pub comment_len: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Literal,
    Folded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chomp {
    Keep,
    Strip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Plus,
    Minus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    BlockHeader { style: Style, indent: Option<u8>, chomp: Option<Chomp> },
    Boolean { value: bool },
    Integer { sign: Option<Sign> },
    LineString,
}

fn classify(text: &str) -> ValueKind {
    debug_assert!(!text.is_empty() && !text.ends_with(' '));
    let bytes = text.as_bytes();
    if let b'|' | b'>' = bytes[0] {
        let style = if bytes[0] == b'|' { Style::Literal } else { Style::Folded };
        let mut i = 1;
        let indent = match bytes.get(i) {
            Some(digit @ b'1'..=b'9') => {
                i += 1;
                Some(digit - b'0')
            }
            _ => None,
        };
        let chomp = match bytes.get(i) {
            Some(b'+') => {
                i += 1;
                Some(Chomp::Keep)
            }
            Some(b'-') => {
                i += 1;
                Some(Chomp::Strip)
            }
            _ => None,
        };
        if i == bytes.len() {
            return ValueKind::BlockHeader { style, indent, chomp };
        }
        return ValueKind::LineString;
    }
    match text {
        "true" => return ValueKind::Boolean { value: true },
        "false" => return ValueKind::Boolean { value: false },
        _ => {}
    }
    let (sign, numeral) = match bytes[0] {
        b'+' => (Some(Sign::Plus), &text[1..]),
        b'-' => (Some(Sign::Minus), &text[1..]),
        _ => (None, text),
    };
    if is_numeral(numeral) {
        return ValueKind::Integer { sign };
    }
    ValueKind::LineString
}

// Deliberately lenient: `0b9` and digitless `0x` are numerals here, because
// the spec makes digit validity an eval concern, not a lexical one.
fn is_numeral(text: &str) -> bool {
    let bytes = text.as_bytes();
    let Some(&first) = bytes.first() else {
        return false;
    };
    if !first.is_ascii_digit() {
        return false;
    }
    let rest = &bytes[1..];
    let digits = match (first, rest.first()) {
        (b'0', Some(b'b' | b'o')) => &rest[1..],
        (b'0', Some(b'x')) => {
            return rest[1..].iter().all(|b| b.is_ascii_hexdigit() || *b == b'_');
        }
        _ => rest,
    };
    digits.iter().all(|b| b.is_ascii_digit() || *b == b'_')
}
