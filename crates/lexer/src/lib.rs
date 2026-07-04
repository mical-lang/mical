#[derive(Debug, Clone)]
pub struct Scanner<'s> {
    source: &'s str,
    pos: usize,
}

impl<'s> Scanner<'s> {
    pub fn new(source: &'s str) -> Self {
        Scanner { source, pos: 0 }
    }

    pub fn next_line(&mut self) -> Option<Line<'s>> {
        let bytes = self.source.as_bytes();
        let start = self.pos;
        if start >= bytes.len() {
            return None;
        }
        let mut indent_end = start;
        while bytes.get(indent_end) == Some(&b' ') {
            indent_end += 1;
        }
        let mut text_end = indent_end;
        while !matches!(bytes.get(text_end), None | Some(b'\n' | b'\r')) {
            text_end += 1;
        }
        let terminator_len = match bytes.get(text_end) {
            None => 0,
            Some(b'\n') => 1,
            Some(b'\r') => {
                let next = bytes.get(text_end + 1);
                if next == Some(&b'\n') { 2 } else { 1 }
            }
            Some(_) => unreachable!(),
        };
        self.pos = text_end + terminator_len;
        Some(Line {
            text: &self.source[start..text_end],
            terminator: &self.source[text_end..self.pos],
            indent_len: indent_end - start,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line<'s> {
    text: &'s str,
    terminator: &'s str,
    indent_len: usize,
}

impl<'s> Line<'s> {
    pub fn text(&self) -> &'s str {
        self.text
    }

    pub fn indent(&self) -> &'s str {
        &self.text[..self.indent_len]
    }

    pub fn content(&self) -> &'s str {
        &self.text[self.indent_len..]
    }

    pub fn terminator(&self) -> &'s str {
        self.terminator
    }

    pub fn head(&self) -> LineHead {
        match self.content().as_bytes().first() {
            None => LineHead::Blank,
            Some(b'\t') => LineHead::Tab,
            Some(b'#') => LineHead::Hash,
            Some(_) => LineHead::Text,
        }
    }

    pub fn is_directive(&self) -> bool {
        let content = self.content().as_bytes();
        self.indent_len == 0
            && content.first() == Some(&b'#')
            && matches!(content.get(1), Some(b) if !matches!(b, b' ' | b'\t'))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineHead {
    Blank,
    Tab,
    Hash,
    Text,
}

macro_rules! debug_assert_line_text {
    ($text:expr) => {
        debug_assert!(
            !$text.bytes().any(|b| matches!(b, b'\n' | b'\r')),
            "input must be text within a single line"
        );
    };
}

mod sealed {
    pub trait Scan<'s>: Sized {
        fn scan(text: &'s str) -> Option<Self>;
    }
}

pub trait Lexeme<'s>: sealed::Scan<'s> {
    fn at(text: &'s str) -> Option<Self> {
        debug_assert_line_text!(text);
        Self::scan(text)
    }

    fn take(rest: &mut &'s str) -> Option<Self> {
        let lexeme = Self::at(rest)?;
        *rest = &rest[lexeme.text().len()..];
        Some(lexeme)
    }

    fn text(&self) -> &'s str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Word<'s>(&'s str);

impl<'s> Word<'s> {
    pub fn text(&self) -> &'s str {
        self.0
    }
}

impl<'s> sealed::Scan<'s> for Word<'s> {
    fn scan(text: &'s str) -> Option<Self> {
        let len = text.bytes().take_while(|b| !matches!(b, b' ' | b'\t')).count();
        (len > 0).then(|| Word(&text[..len]))
    }
}

impl<'s> Lexeme<'s> for Word<'s> {
    fn text(&self) -> &'s str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quote {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quoted<'s> {
    text: &'s str,
    quote: Quote,
    is_closed: bool,
}

impl<'s> Quoted<'s> {
    pub fn text(&self) -> &'s str {
        self.text
    }

    pub fn quote(&self) -> Quote {
        self.quote
    }

    pub fn content(&self) -> &'s str {
        &self.text[1..self.text.len() - self.is_closed as usize]
    }

    pub fn is_closed(&self) -> bool {
        self.is_closed
    }
}

impl<'s> sealed::Scan<'s> for Quoted<'s> {
    fn scan(text: &'s str) -> Option<Self> {
        let bytes = text.as_bytes();
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
                    return Some(Quoted { text: &text[..i + 1], quote, is_closed: true });
                }
                _ => i += 1,
            }
        }
        Some(Quoted { text, quote, is_closed: false })
    }
}

impl<'s> Lexeme<'s> for Quoted<'s> {
    fn text(&self) -> &'s str {
        self.text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Separator<'s> {
    text: &'s str,
    spaces_len: usize,
}

impl<'s> Separator<'s> {
    pub fn text(&self) -> &'s str {
        self.text
    }

    pub fn spaces(&self) -> &'s str {
        &self.text[..self.spaces_len]
    }

    pub fn tab_run(&self) -> Option<&'s str> {
        (self.spaces_len < self.text.len()).then(|| &self.text[self.spaces_len..])
    }
}

impl<'s> sealed::Scan<'s> for Separator<'s> {
    fn scan(text: &'s str) -> Option<Self> {
        let bytes = text.as_bytes();
        let mut spaces_len = 0;
        while bytes.get(spaces_len) == Some(&b' ') {
            spaces_len += 1;
        }
        let mut end = spaces_len;
        if bytes.get(end) == Some(&b'\t') {
            while matches!(bytes.get(end), Some(b' ' | b'\t')) {
                end += 1;
            }
        }
        (end > 0).then(|| Separator { text: &text[..end], spaces_len })
    }
}

impl<'s> Lexeme<'s> for Separator<'s> {
    fn text(&self) -> &'s str {
        self.text
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trailing<'s> {
    Spaces(&'s str),
    Comment(&'s str),
}

impl<'s> Trailing<'s> {
    pub fn split(text: &'s str) -> (&'s str, Option<Trailing<'s>>) {
        debug_assert_line_text!(text);
        let bytes = text.as_bytes();
        let hash = (0..bytes.len()).find(|&i| bytes[i] == b'#' && (i == 0 || bytes[i - 1] == b' '));
        match hash {
            Some(hash) => {
                let mut start = hash;
                while start > 0 && bytes[start - 1] == b' ' {
                    start -= 1;
                }
                (&text[..start], Some(Trailing::Comment(&text[start..])))
            }
            None => {
                let mut end = bytes.len();
                while end > 0 && bytes[end - 1] == b' ' {
                    end -= 1;
                }
                if end == bytes.len() {
                    (text, None)
                } else {
                    (&text[..end], Some(Trailing::Spaces(&text[end..])))
                }
            }
        }
    }

    pub fn text(&self) -> &'s str {
        match self {
            Trailing::Spaces(text) | Trailing::Comment(text) => text,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    BlockHeader { style: Style, indent: Option<u8>, chomp: Option<Chomp> },
    Boolean { value: bool },
    Integer { sign: Option<Sign>, radix: Radix },
    LineString,
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
pub enum Radix {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

impl ValueKind {
    pub fn classify(content: &str) -> Self {
        debug_assert!(
            Trailing::split(content).1.is_none(),
            "value content must be pre-split from its trailing part"
        );
        let bytes = content.as_bytes();
        let Some(&first) = bytes.first() else {
            return ValueKind::LineString;
        };
        if let b'|' | b'>' = first {
            let style = if first == b'|' { Style::Literal } else { Style::Folded };
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
        match content {
            "true" => return ValueKind::Boolean { value: true },
            "false" => return ValueKind::Boolean { value: false },
            _ => {}
        }
        let (sign, numeral) = match first {
            b'+' => (Some(Sign::Plus), &content[1..]),
            b'-' => (Some(Sign::Minus), &content[1..]),
            _ => (None, content),
        };
        if let Some(radix) = numeral_radix(numeral) {
            return ValueKind::Integer { sign, radix };
        }
        ValueKind::LineString
    }
}

fn numeral_radix(text: &str) -> Option<Radix> {
    let bytes = text.as_bytes();
    let (&first, rest) = bytes.split_first()?;
    if !first.is_ascii_digit() {
        return None;
    }
    fn each_digits(digits: &[u8], is_digit: impl Fn(u8) -> bool) -> bool {
        let mut has_digit = false;
        for &b in digits {
            if b == b'_' {
                continue;
            }
            if !is_digit(b) {
                return false;
            }
            has_digit = true;
        }
        has_digit
    }
    match (first, rest.first()) {
        (b'0', Some(b'b')) => {
            each_digits(&rest[1..], |b| matches!(b, b'0' | b'1')).then_some(Radix::Binary)
        }
        (b'0', Some(b'o')) => {
            each_digits(&rest[1..], |b| matches!(b, b'0'..=b'7')).then_some(Radix::Octal)
        }
        (b'0', Some(b'x')) => {
            each_digits(&rest[1..], |b| b.is_ascii_hexdigit()).then_some(Radix::Hexadecimal)
        }
        // The leading digit already makes it a numeral, so the rest may be empty.
        _ => {
            let is_decimal = rest.iter().all(|&b| b == b'_' || b.is_ascii_digit());
            is_decimal.then_some(Radix::Decimal)
        }
    }
}
