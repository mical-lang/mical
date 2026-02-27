use crate::{
    Error, ValueRaw,
    text_arena::{TextArena, TextId},
};
use mical_cli_syntax::{
    SyntaxKind,
    ast::{self, BooleanKind},
};

mod joined_str;
use joined_str::*;

mod temporary_string;
use temporary_string::*;

mod unescape;
use unescape::*;

pub(crate) struct Output {
    pub(crate) arena: TextArena,
    pub(crate) entries: Vec<(TextId, ValueRaw)>,
    pub(crate) errors: Vec<Error>,
}

pub(crate) fn eval_source_file(source_file: &ast::SourceFile) -> Output {
    let mut ctx = Context::new();
    source_file.eval(&mut ctx);
    ctx.finish()
}

struct Context {
    arena: TextArena,
    entries: Vec<(TextId, ValueRaw)>,
    prefix: String,
    temporary_string: TemporaryString,
    errors: Vec<Error>,
}

impl Context {
    fn new() -> Self {
        Context {
            arena: TextArena::new(),
            entries: Vec::new(),
            prefix: String::new(),
            temporary_string: TemporaryString::new(),
            errors: Vec::new(),
        }
    }

    fn finish(self) -> Output {
        Output { arena: self.arena, entries: self.entries, errors: self.errors }
    }
}

trait Eval {
    type Output;
    fn eval(&self, ctx: &mut Context) -> Self::Output;
}

impl Eval for ast::SourceFile {
    type Output = ();

    fn eval(&self, ctx: &mut Context) {
        for item in self.items() {
            item.eval(ctx);
        }
    }
}

impl Eval for ast::Item {
    type Output = ();

    fn eval(&self, ctx: &mut Context) {
        match self {
            ast::Item::Entry(entry) => entry.eval(ctx),
            ast::Item::PrefixBlock(block) => block.eval(ctx),
            ast::Item::Directive(_) => {}
        }
    }
}

impl Eval for ast::Entry {
    type Output = ();

    fn eval(&self, ctx: &mut Context) {
        let Some(key) = self.key() else { return };
        let Some(value) = self.value() else { return };

        let key_id = {
            let full_key = match key {
                ast::Key::Word(word_key) => {
                    let Some(token) = word_key.word() else { return };
                    ctx.prefix.joined(token.text())
                }
                ast::Key::Quoted(quoted_key) => {
                    let Some(string) = quoted_key.string() else { return };
                    let espaced: &mut String = ctx.temporary_string.get();
                    unescape(string.text(), espaced, string.text_range().start(), &mut ctx.errors);
                    ctx.prefix.joined(espaced)
                }
            };
            ctx.arena.alloc(&full_key)
        };

        let Some(value_raw) = value.eval(ctx) else { return };
        ctx.entries.push((key_id, value_raw));
    }
}

impl Eval for ast::PrefixBlock {
    type Output = ();

    fn eval(&self, ctx: &mut Context) {
        let Some(key) = self.key() else { return };

        let prev_prefix_len = ctx.prefix.len();

        match key {
            ast::Key::Word(word_key) => {
                let Some(token) = word_key.word() else { return };
                ctx.prefix.push_str(token.text());
            }
            ast::Key::Quoted(quoted_key) => {
                let Some(string) = quoted_key.string() else { return };
                let espaced: &mut String = ctx.temporary_string.get();
                unescape(string.text(), espaced, string.text_range().start(), &mut ctx.errors);
                ctx.prefix.push_str(espaced);
            }
        };

        for item in self.items() {
            item.eval(ctx);
        }

        ctx.prefix.truncate(prev_prefix_len);
    }
}

impl Eval for ast::Value {
    type Output = Option<ValueRaw>;

    fn eval(&self, ctx: &mut Context) -> Option<ValueRaw> {
        let value = match self {
            ast::Value::Boolean(b) => {
                let val = b.eval(ctx)?;
                ValueRaw::Bool(val)
            }
            ast::Value::Integer(i) => {
                let text_id = i.eval(ctx)?;
                ValueRaw::Integer(text_id)
            }
            ast::Value::LineString(ls) => {
                let string = ls.string()?;
                let text = string.text();
                let text_id = ctx.arena.alloc(text);
                ValueRaw::String(text_id)
            }
            ast::Value::QuotedString(qs) => {
                let text_id = qs.eval(ctx)?;
                ValueRaw::String(text_id)
            }
            ast::Value::BlockString(bs) => {
                let text_id = bs.eval(ctx)?;
                ValueRaw::String(text_id)
            }
        };
        Some(value)
    }
}

impl Eval for ast::Boolean {
    type Output = Option<bool>;

    fn eval(&self, _ctx: &mut Context) -> Self::Output {
        let val = match self.kind()? {
            BooleanKind::True => true,
            BooleanKind::False => false,
        };
        Some(val)
    }
}

impl Eval for ast::Integer {
    type Output = Option<TextId>;

    fn eval(&self, ctx: &mut Context) -> Self::Output {
        let buf = ctx.temporary_string.get();
        if let Some(sign) = self.sign() {
            buf.push_str(sign.text());
        }
        buf.push_str(self.numeral()?.text());
        Some(ctx.arena.alloc(buf))
    }
}

impl Eval for ast::QuotedString {
    type Output = Option<TextId>;

    fn eval(&self, ctx: &mut Context) -> Self::Output {
        let string = self.string()?;
        let buf = ctx.temporary_string.get();
        unescape(string.text(), buf, string.text_range().start(), &mut ctx.errors);
        Some(ctx.arena.alloc(buf))
    }
}

impl Eval for ast::BlockString {
    type Output = Option<TextId>;

    fn eval(&self, ctx: &mut Context) -> Self::Output {
        let (is_folded, chomp) = match self.header() {
            Some(h) => {
                let is_folded = h.style().is_some_and(|s| s.kind() == SyntaxKind::GT);
                let chomp = h.chomp().map(|c| c.kind());
                (is_folded, chomp)
            }
            None => (false, None),
        };

        let buf = ctx.temporary_string.get();
        let mut has_lines = false;

        if is_folded {
            let mut prev_content: Option<bool> = None;
            for line in self.lines() {
                has_lines = true;
                match line.string() {
                    Some(token) => {
                        let text = token.text();
                        let more_indented = text.starts_with(' ');
                        if let Some(prev_was_more) = prev_content {
                            if prev_was_more || more_indented {
                                buf.push('\n');
                            } else {
                                buf.push(' ');
                            }
                        }
                        buf.push_str(text);
                        prev_content = Some(more_indented);
                    }
                    None => {
                        buf.push('\n');
                        prev_content = None;
                    }
                }
            }
        } else {
            for (i, line) in self.lines().enumerate() {
                has_lines = true;
                if i > 0 {
                    buf.push('\n');
                }
                if let Some(token) = line.string() {
                    buf.push_str(token.text());
                }
            }
        }

        if !has_lines {
            return Some(ctx.arena.alloc(""));
        }

        buf.push('\n');

        match chomp {
            Some(SyntaxKind::MINUS) => {
                let end = buf.trim_end_matches('\n').len();
                buf.truncate(end);
            }
            Some(SyntaxKind::PLUS) => {}
            _ => {
                let end = buf.trim_end_matches('\n').len();
                buf.truncate(end);
                buf.push('\n');
            }
        }

        Some(ctx.arena.alloc(buf))
    }
}
