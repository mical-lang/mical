use std::borrow::Cow;

use mical_cli_syntax::{GreenNode, SyntaxError, SyntaxKind, token::TokenStream};
use rowan::{GreenNodeBuilder, TextRange};

mod event;
mod grammar;
mod parser;
mod token_set;

use event::Event;
use parser::Parser;

pub fn parse<'s>(token_stream: impl TokenStream<'s>) -> (GreenNode, Vec<SyntaxError>) {
    let source = token_stream.source();
    let events = {
        let mut parser = Parser::new(token_stream);
        grammar::source_file(&mut parser);
        parser.finish()
    };
    let mut builder = NodeBuilder::new(source);
    // let mut forward_parents = Vec::new();
    // for i in 0..events.len() {
    //     match events.take(i) {
    //         Event::StartNode { kind, forward_parent } => {
    //             if forward_parent.is_none() {
    //                 builder.start_node(kind);
    //                 continue;
    //             }
    //             forward_parents.push(kind);
    //             let mut idx = i;
    //             let mut fp = forward_parent;
    //             while let Some(fpi) = fp {
    //                 idx += fpi.get() as usize;
    //                 fp = match events.take(idx) {
    //                     Event::StartNode { kind, forward_parent } => {
    //                         forward_parents.push(kind);
    //                         forward_parent
    //                     }
    //                     _ => unreachable!(),
    //                 };
    //             }
    //             for kind in forward_parents.drain(..).rev() {
    //                 builder.start_node(kind);
    //             }
    //         }
    //         Event::FinishNode => builder.finish_node(),
    //         Event::Token { kind, len } => builder.token(kind, len),
    //         Event::Error { message } => builder.error(message),
    //     }
    // }
    // let mut builder = GreenNodeBuilder::new();
    // let mut errors = Vec::new();
    // let mut offset = 0;
    for event in events {
        match event {
            Event::StartNode { kind } => builder.start_node(kind),
            Event::FinishNode => builder.finish_node(),
            Event::Token { kind, len } => builder.token(kind, len),
            Event::Error { message } => builder.error(message),
        }
    }
    builder.finish()
}

struct NodeBuilder<'s> {
    source: &'s str,
    builder: GreenNodeBuilder<'s>,
    errors: Vec<SyntaxError>,
    offset: u32,
}

impl<'s> NodeBuilder<'s> {
    fn new(source: &'s str) -> Self {
        NodeBuilder { source, builder: GreenNodeBuilder::new(), errors: Vec::new(), offset: 0 }
    }

    fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind.into());
    }

    fn finish_node(&mut self) {
        self.builder.finish_node();
    }

    fn token(&mut self, kind: SyntaxKind, len: u32) {
        let text = &self.source[(self.offset as usize)..(self.offset + len) as usize];
        self.builder.token(kind.into(), text);
        self.offset += len;
    }

    fn error(&mut self, message: impl Into<Cow<'static, str>>) {
        let range = TextRange::empty(self.offset.into());
        self.errors.push(SyntaxError::new(message, range));
    }

    fn finish(self) -> (GreenNode, Vec<SyntaxError>) {
        (self.builder.finish(), self.errors)
    }
}
