use crate::{
    event::{Event, EventContainer},
    token_set::TokenSet,
};
use mical_cli_syntax::{
    SyntaxKind, T,
    token::{Quote, TokenKind, TokenStream},
};
use std::{borrow::Cow, mem};

pub(crate) struct Parser {
    kinds: Vec<SyntaxKind>,
    lens: Vec<u32>,
    pos: usize,
    events: EventContainer,
}

impl Parser {
    pub(crate) fn new<'s>(token_stream: impl TokenStream<'s>) -> Self {
        let mut kinds = Vec::new();
        let mut lens = Vec::new();
        let mut push = |kind: SyntaxKind, len: u32| {
            kinds.push(kind);
            lens.push(len);
        };
        for token in token_stream {
            match token.kind {
                TokenKind::Word => push(T![word], token.len),
                TokenKind::True => push(T![true], token.len),
                TokenKind::False => push(T![false], token.len),
                TokenKind::Tab => push(T!['\t'], token.len),
                TokenKind::Newline => push(T!['\n'], token.len),
                TokenKind::Space => push(T![' '], token.len),
                TokenKind::CloseBrace => push(T!['}'], token.len),
                TokenKind::Greater => push(T![>], token.len),
                TokenKind::Minus => push(T![-], token.len),
                TokenKind::OpenBrace => push(T!['{'], token.len),
                TokenKind::Pipe => push(T![|], token.len),
                TokenKind::Plus => push(T![+], token.len),
                TokenKind::Sharp => push(T![#], token.len),
                TokenKind::Numeral { radix: _, is_empty } => {
                    if is_empty {
                        push(T![word], token.len);
                    } else {
                        push(T![numeral], token.len);
                    }
                }
                TokenKind::String { is_terminated, quote } => {
                    let quote_kind = match quote {
                        Quote::Single => T!['\''],
                        Quote::Double => T!['"'],
                    };
                    push(quote_kind, 1);
                    if is_terminated {
                        push(T![string], token.len - 2);
                        push(quote_kind, 1);
                    } else {
                        push(T![string], token.len - 1);
                    }
                }
            };
        }
        Parser { kinds, lens, pos: 0, events: EventContainer::new() }
    }

    pub(crate) fn current(&self) -> Option<SyntaxKind> {
        self.kinds.get(self.pos).copied()
    }

    pub(crate) fn current_len(&self) -> Option<u32> {
        self.lens.get(self.pos).copied()
    }

    pub(crate) fn at(&self, kind: SyntaxKind) -> bool {
        self.kinds.get(self.pos) == Some(&kind)
    }

    pub(crate) fn at_ts(&self, kinds: TokenSet) -> bool {
        let Some(current) = self.current() else {
            return false;
        };
        kinds.contains(current)
    }

    pub(crate) fn at_eof(&self) -> bool {
        self.current().is_none()
    }

    pub(crate) fn nth_at(&self, n: usize, kind: SyntaxKind) -> bool {
        self.kinds.get(self.pos + n) == Some(&kind)
    }

    pub(crate) fn nth_at_ts(&self, n: usize, kinds: TokenSet) -> bool {
        let Some(current) = self.kinds.get(self.pos + n).copied() else {
            return false;
        };
        kinds.contains(current)
    }

    pub(crate) fn nth_at_eof(&self, n: usize) -> bool {
        self.kinds.get(self.pos + n).is_none()
    }

    pub(crate) fn nth_len(&self, n: usize) -> Option<u32> {
        self.lens.get(self.pos + n).copied()
    }

    pub(crate) fn start(&mut self) -> Marker {
        let pos = self.events.len() as u32;
        self.events.push_tombstone();
        Marker { pos }
    }

    pub(crate) fn eat(&mut self, kind: SyntaxKind) -> bool {
        if !self.at(kind) {
            return false;
        }
        let Some(len) = self.current_len() else {
            unreachable!("Unexpected EOF");
        };
        self.events.push(Event::Token { kind, len });
        self.pos += 1;
        true
    }

    pub(crate) fn eat_upto(&mut self, kind: SyntaxKind, len: u32) -> bool {
        if !self.at(kind) {
            return false;
        }
        let Some(current_len) = self.current_len() else {
            unreachable!("Unexpected EOF");
        };
        if current_len < len {
            return false;
        }
        self.events.push(Event::Token { kind, len });
        if current_len > len {
            self.lens[self.pos] -= len;
        } else {
            self.pos += 1;
        }
        true
    }

    pub(crate) fn bump(&mut self, kind: SyntaxKind) {
        assert!(self.eat(kind));
    }

    pub(crate) fn bump_upto(&mut self, kind: SyntaxKind, len: u32) {
        assert!(self.eat_upto(kind, len));
    }

    pub(crate) fn bump_any(&mut self) {
        let Some(kind) = self.current() else {
            panic!("Unexpected EOF");
        };
        let Some(len) = self.current_len() else {
            panic!("Unexpected EOF");
        };
        self.events.push(Event::Token { kind, len });
        self.pos += 1;
    }

    pub(crate) fn bump_remap(&mut self, kind: SyntaxKind, n: usize) {
        if self.pos + n > self.kinds.len() {
            panic!("Unexpected EOF");
        }
        let len = self.lens[self.pos..self.pos + n].iter().sum();
        self.events.push(Event::Token { kind, len });
        self.pos += n;
    }

    pub(crate) fn error(&mut self, message: impl Into<Cow<'static, str>>) {
        let message = message.into();
        self.events.push(Event::Error { message });
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
        // CompletedMarker { pos }
    }
}

impl Drop for Marker {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            panic!("Marker must be completed")
        }
    }
}

// pub(crate) struct CompletedMarker {
//     pos: u32,
// }
//
// impl CompletedMarker {
//     pub(crate) fn precede(self, p: &mut Parser) -> Marker {
//         let new_pos = p.start();
//         let idx = self.pos as usize;
//         p.events.set_forward_parent(idx, new_pos.pos - self.pos);
//         new_pos
//     }
// }
