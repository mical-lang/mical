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
                let text = bs.eval(ctx);
                let id = ctx.arena.alloc(&text);
                ValueRaw::String(id)
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
        let text = ctx.temporary_string.get();
        if let Some(sign) = self.sign() {
            text.push_str(sign.text());
        }
        text.push_str(self.numeral()?.text());
        Some(ctx.arena.alloc(text))
    }
}

impl Eval for ast::QuotedString {
    type Output = Option<TextId>;

    fn eval(&self, ctx: &mut Context) -> Self::Output {
        let string = self.string()?;
        let text = ctx.temporary_string.get();
        unescape(string.text(), text, string.text_range().start(), &mut ctx.errors);
        Some(ctx.arena.alloc(text))
    }
}

impl Eval for ast::BlockString {
    type Output = String;

    fn eval(&self, _ctx: &mut Context) -> Self::Output {
        let (is_folded, chomp) = match self.header() {
            Some(h) => {
                let is_folded = h.style().is_some_and(|s| s.kind() == SyntaxKind::GT);
                let chomp = h.chomp().map(|c| c.kind());
                (is_folded, chomp)
            }
            None => (false, None),
        };

        let lines: Vec<Option<String>> =
            self.lines().map(|line| line.string().map(|t| t.text().to_string())).collect();

        if lines.is_empty() {
            return String::new();
        }

        let raw = if is_folded { fold_lines(&lines) } else { literal_lines(&lines) };
        apply_chomp(&raw, chomp)
    }
}

/// Literal style (`|`): newlines between lines are preserved as-is.
fn literal_lines(lines: &[Option<String>]) -> String {
    let mut result = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        if let Some(text) = line {
            result.push_str(text);
        }
    }
    result.push('\n');
    result
}

/// Folded style (`>`): single newlines between adjacent content lines are
/// replaced by spaces.  Empty lines produce `\n` in the output.
fn fold_lines(lines: &[Option<String>]) -> String {
    let mut result = String::new();
    let mut prev_was_content = false;
    for line in lines {
        match line {
            Some(text) => {
                if prev_was_content {
                    result.push(' ');
                }
                result.push_str(text);
                prev_was_content = true;
            }
            None => {
                result.push('\n');
                prev_was_content = false;
            }
        }
    }
    result.push('\n');
    result
}

fn apply_chomp(raw: &str, chomp: Option<SyntaxKind>) -> String {
    match chomp {
        Some(SyntaxKind::MINUS) => raw.trim_end_matches('\n').to_string(),
        Some(SyntaxKind::PLUS) => raw.to_string(),
        _ => {
            let trimmed = raw.trim_end_matches('\n');
            format!("{trimmed}\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_lines_basic() {
        let lines = vec![Some("a".into()), Some("b".into())];
        assert_eq!(literal_lines(&lines), "a\nb\n");
    }

    #[test]
    fn literal_lines_with_empty() {
        let lines = vec![Some("a".into()), None, Some("b".into())];
        assert_eq!(literal_lines(&lines), "a\n\nb\n");
    }

    #[test]
    fn literal_lines_trailing_empty() {
        let lines = vec![Some("x".into()), None, None];
        assert_eq!(literal_lines(&lines), "x\n\n\n");
    }

    #[test]
    fn fold_lines_basic() {
        let lines = vec![
            Some("This is a long".into()),
            Some("sentence split".into()),
            Some("over lines.".into()),
            None,
            Some("New paragraph.".into()),
        ];
        assert_eq!(
            fold_lines(&lines),
            "This is a long sentence split over lines.\nNew paragraph.\n"
        );
    }

    #[test]
    fn fold_lines_multiple_empty() {
        let lines = vec![Some("a".into()), None, None, Some("b".into())];
        assert_eq!(fold_lines(&lines), "a\n\nb\n");
    }

    #[test]
    fn chomp_clip() {
        assert_eq!(apply_chomp("hello\nworld\n", None), "hello\nworld\n");
        assert_eq!(apply_chomp("hello\nworld\n\n", None), "hello\nworld\n");
        assert_eq!(apply_chomp("a\n\n\n", None), "a\n");
    }

    #[test]
    fn chomp_strip() {
        assert_eq!(apply_chomp("hello\nworld\n", Some(SyntaxKind::MINUS)), "hello\nworld");
        assert_eq!(apply_chomp("hello\nworld\n\n", Some(SyntaxKind::MINUS)), "hello\nworld");
    }

    #[test]
    fn chomp_keep() {
        assert_eq!(apply_chomp("hello\n\n\n", Some(SyntaxKind::PLUS)), "hello\n\n\n");
        assert_eq!(apply_chomp("x\n", Some(SyntaxKind::PLUS)), "x\n");
    }
}
