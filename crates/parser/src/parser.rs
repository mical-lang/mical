use crate::event::{Event, EventContainer};
use mical_cli_lexer::{
    Line, LineHead, LineScanner, QuotedToken, Separator, SplitComment, ValueKind, WhitespaceRun,
};
use mical_cli_syntax::{SyntaxKind, T};
use std::mem;

pub(crate) struct Parser<'s> {
    lines: Vec<Line<'s>>,
    line_cursor: usize,
    scanner: LineScanner<'s>,
    pub(crate) last_item_indent: u32,
    events: EventContainer,
}

impl<'s> Parser<'s> {
    pub(crate) fn new(source: &'s str) -> Self {
        let lines: Vec<Line<'s>> = mical_cli_lexer::scan_lines(source).collect();
        let scanner = lines.first().map_or_else(LineScanner::empty, |line| line.scan());
        Parser {
            lines,
            line_cursor: 0,
            scanner,
            last_item_indent: 0,
            events: EventContainer::new(),
        }
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

    pub(crate) fn indent(&self) -> u32 {
        self.current_line().indent()
    }

    pub(crate) fn is_directive(&self) -> bool {
        self.current_line().is_directive()
    }

    // Scanning never consumes; the only ways to consume text are `token` and
    // `finish_line`, which keep the emitted events and the consumed text in
    // lockstep.
    pub(crate) fn rest_len(&self) -> u32 {
        self.scanner.rest_len()
    }

    pub(crate) fn scan_word(&self) -> u32 {
        self.scanner.scan_word()
    }

    pub(crate) fn scan_quoted(&self) -> Option<QuotedToken> {
        self.scanner.scan_quoted()
    }

    pub(crate) fn scan_separator(&self) -> Separator {
        self.scanner.scan_separator()
    }

    pub(crate) fn scan_whitespace_run(&self) -> Option<WhitespaceRun> {
        self.scanner.scan_whitespace_run()
    }

    pub(crate) fn split_comment(&self) -> SplitComment {
        self.scanner.split_comment()
    }

    pub(crate) fn classify_value(&self, value_len: u32) -> ValueKind {
        self.scanner.classify_value(value_len)
    }

    pub(crate) fn token(&mut self, kind: SyntaxKind, len: u32) {
        debug_assert!(len > 0, "zero-length token");
        self.scanner.advance(len);
        self.events.push(Event::Token { kind, len });
    }

    pub(crate) fn finish_line(&mut self) {
        self.line_cursor += 1;
        let next =
            self.lines.get(self.line_cursor).map_or_else(LineScanner::empty, |line| line.scan());
        let finished = mem::replace(&mut self.scanner, next);
        let terminator_len = finished.take_terminator();
        if terminator_len > 0 {
            self.events.push(Event::Token { kind: T!['\n'], len: terminator_len });
        }
    }

    // The error's range is the next `len` bytes; it is resolved to absolute
    // positions when the events are replayed, so the parser itself never
    // deals with source offsets.
    pub(crate) fn error(&mut self, message: &'static str, len: u32) {
        debug_assert!(len <= self.scanner.rest_len(), "error range exceeds the current line");
        self.events.push(Event::Error { message, len });
    }

    pub(crate) fn start(&mut self) -> Marker {
        let pos = self.events.len() as u32;
        self.events.push_tombstone();
        Marker { pos }
    }

    pub(crate) fn finish(self) -> EventContainer {
        self.events
    }
}

#[must_use]
pub(crate) struct Marker {
    pos: u32,
}

impl Marker {
    pub(crate) fn complete(self, p: &mut Parser, kind: SyntaxKind) {
        let pos = self.pos;
        mem::forget(self);
        p.events.replace_tombstone(pos as usize, Event::StartNode { kind });
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
