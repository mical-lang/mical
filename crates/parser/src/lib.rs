use mical_cli_syntax::{GreenNode, SyntaxError, SyntaxKind, TextRange};
use rowan::GreenNodeBuilder;

mod event;
mod grammar;
mod parser;

use event::Event;
use parser::Parser;

pub fn parse(source: &str) -> (GreenNode, Vec<SyntaxError>) {
    let events = {
        let mut parser = Parser::new(source);
        grammar::source_file(&mut parser);
        parser.finish()
    };
    let mut builder = NodeBuilder::new(source);
    let mut errors = Vec::new();
    for event in events {
        match event {
            Event::StartNode { kind } => builder.start_node(kind),
            Event::FinishNode => builder.finish_node(),
            Event::Token { kind, len } => builder.token(kind, len),
            Event::Error { message, len } => {
                let start = builder.offset();
                let range = TextRange::new(start.into(), (start + len).into());
                errors.push(SyntaxError::new(message, range));
            }
        }
    }
    (builder.finish(), errors)
}

struct NodeBuilder<'s> {
    source: &'s str,
    builder: GreenNodeBuilder<'s>,
    offset: u32,
}

impl<'s> NodeBuilder<'s> {
    fn new(source: &'s str) -> Self {
        NodeBuilder { source, builder: GreenNodeBuilder::new(), offset: 0 }
    }

    fn offset(&self) -> u32 {
        self.offset
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

    fn finish(self) -> GreenNode {
        debug_assert_eq!(self.offset as usize, self.source.len(), "CST does not cover the source");
        self.builder.finish()
    }
}
