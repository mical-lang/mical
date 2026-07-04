use crate::event::{Event, EventContainer};
use core::{iter, mem};
use mical_cli_lexer::{
    Lexeme as _, Line, LineHead, Quoted, Scanner, Separator, Trailing, ValueKind, Word,
};
use mical_cli_syntax::{SyntaxKind, T};

pub(crate) struct Parser<'s> {
    lines: Vec<Line<'s>>,
    line_cursor: usize,
    rest: &'s str,
    pub(crate) last_item_indent: usize,
    events: EventContainer,
}

impl<'s> Parser<'s> {
    pub(crate) fn new(source: &'s str) -> Self {
        let mut scanner = Scanner::new(source);
        let lines = iter::from_fn(|| scanner.next_line()).collect::<Vec<_>>();
        let rest = lines.first().map_or("", |line| line.text());
        Parser { lines, line_cursor: 0, rest, last_item_indent: 0, events: EventContainer::new() }
    }

    pub(crate) fn at_eof(&self) -> bool {
        self.line_cursor >= self.lines.len()
    }

    pub(crate) fn nth_line(&self, n: usize) -> Option<Line<'s>> {
        self.lines.get(self.line_cursor + n).copied()
    }

    fn current_line(&self) -> Line<'s> {
        self.lines[self.line_cursor]
    }

    pub(crate) fn head(&self) -> LineHead {
        self.current_line().head()
    }

    pub(crate) fn indent(&self) -> usize {
        self.current_line().indent().len()
    }

    pub(crate) fn is_directive(&self) -> bool {
        self.current_line().is_directive()
    }

    pub(crate) fn rest_len(&self) -> usize {
        self.rest.len()
    }

    pub(crate) fn word(&self) -> Option<Word<'s>> {
        Word::at(self.rest)
    }

    pub(crate) fn quoted(&self) -> Option<Quoted<'s>> {
        Quoted::at(self.rest)
    }

    pub(crate) fn separator(&self) -> Option<Separator<'s>> {
        Separator::at(self.rest)
    }

    pub(crate) fn trailing(&self) -> (&'s str, Option<Trailing<'s>>) {
        Trailing::split(self.rest)
    }

    pub(crate) fn classify_value(&self, content: &str) -> ValueKind {
        ValueKind::classify(content)
    }

    pub(crate) fn token(&mut self, kind: SyntaxKind, len: usize) {
        debug_assert!(len > 0, "zero-length token");
        assert!(len <= self.rest.len(), "token exceeds the end of the line");
        self.rest = &self.rest[len..];
        self.events.push(Event::Token { kind, len: len as u32 });
    }

    pub(crate) fn finish_line(&mut self) {
        assert!(self.rest.is_empty(), "cannot finish line with unconsumed text");
        let terminator = self.current_line().terminator();
        if !terminator.is_empty() {
            self.events.push(Event::Token { kind: T!['\n'], len: terminator.len() as u32 });
        }
        self.line_cursor += 1;
        self.rest = self.lines.get(self.line_cursor).map_or("", |line| line.text());
    }

    pub(crate) fn error(&mut self, message: &'static str, len: usize) {
        debug_assert!(len <= self.rest.len(), "error range exceeds the current line");
        self.events.push(Event::Error { message, len: len as u32 });
    }

    pub(crate) fn start(&mut self) -> Marker {
        let pos = self.events.len();
        self.events.push_tombstone();
        Marker { pos }
    }

    pub(crate) fn finish(self) -> EventContainer {
        self.events
    }
}

#[must_use]
pub(crate) struct Marker {
    pos: usize,
}

impl Marker {
    pub(crate) fn complete(self, p: &mut Parser, kind: SyntaxKind) {
        let pos = self.pos;
        mem::forget(self);
        p.events.replace_tombstone(pos, Event::StartNode { kind });
        p.events.push(Event::FinishNode);
    }
}

impl Drop for Marker {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            panic!("Marker must be completed")
        }
    }
}
