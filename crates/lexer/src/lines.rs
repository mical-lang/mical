use crate::scanner::LineScanner;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineHead {
    Blank,
    Tab,
    Hash,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub struct Line<'s> {
    pub(crate) text: &'s str,
    pub(crate) terminator_len: u32,
    indent: u32,
    head: LineHead,
}

impl<'s> Line<'s> {
    pub fn text(&self) -> &'s str {
        self.text
    }

    pub fn text_len(&self) -> u32 {
        self.text.len() as u32
    }

    pub fn terminator_len(&self) -> u32 {
        self.terminator_len
    }

    pub fn indent(&self) -> u32 {
        self.indent
    }

    pub fn head(&self) -> LineHead {
        self.head
    }

    pub fn is_directive(&self) -> bool {
        let bytes = self.text.as_bytes();
        self.indent == 0
            && bytes.first() == Some(&b'#')
            && matches!(bytes.get(1), Some(b) if !matches!(b, b' ' | b'\t'))
    }

    pub fn scan(self) -> LineScanner<'s> {
        LineScanner::new(self)
    }
}

pub fn scan_lines(source: &str) -> Lines<'_> {
    assert!(source.len() <= u32::MAX as usize, "source code is too large");
    Lines { source, pos: 0 }
}

#[derive(Debug, Clone)]
pub struct Lines<'s> {
    source: &'s str,
    pos: usize,
}

impl<'s> Iterator for Lines<'s> {
    type Item = Line<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.source.as_bytes();
        let start = self.pos;
        if start >= bytes.len() {
            return None;
        }
        let mut text_end = start;
        while text_end < bytes.len() && !matches!(bytes[text_end], b'\n' | b'\r') {
            text_end += 1;
        }
        let terminator_len = match bytes.get(text_end) {
            None => 0,
            Some(b'\n') => 1,
            Some(_cr) if bytes.get(text_end + 1) == Some(&b'\n') => 2,
            Some(_cr) => 1,
        };
        let mut first_non_space = start;
        while first_non_space < text_end && bytes[first_non_space] == b' ' {
            first_non_space += 1;
        }
        let head = if first_non_space == text_end {
            LineHead::Blank
        } else {
            match bytes[first_non_space] {
                b'\t' => LineHead::Tab,
                b'#' => LineHead::Hash,
                _ => LineHead::Other,
            }
        };
        self.pos = text_end + terminator_len;
        Some(Line {
            text: &self.source[start..text_end],
            terminator_len: terminator_len as u32,
            indent: (first_non_space - start) as u32,
            head,
        })
    }
}

impl std::iter::FusedIterator for Lines<'_> {}
