use mical_syntax::ast::{self, AstNode, BooleanKind};
use mical_syntax::{SyntaxKind, TextRange, TextSize};

use crate::ValueRaw;
use crate::error::ConfigError;
use crate::text_arena::{TextArena, TextId};

pub(crate) struct EvalContext {
    pub(crate) arena: TextArena,
    pub(crate) entries: Vec<(TextId, ValueRaw)>,
    pub(crate) errors: Vec<ConfigError>,
    prefix: String,
}

impl EvalContext {
    pub(crate) fn new() -> Self {
        Self {
            arena: TextArena::new(),
            entries: Vec::new(),
            errors: Vec::new(),
            prefix: String::new(),
        }
    }
}

trait Eval {
    type Output;
    fn eval(&self, ctx: &mut EvalContext) -> Self::Output;
}

impl Eval for ast::SourceFile {
    type Output = ();
    fn eval(&self, ctx: &mut EvalContext) {
        for item in self.items() {
            item.eval(ctx);
        }
    }
}

impl Eval for ast::Item {
    type Output = ();
    fn eval(&self, ctx: &mut EvalContext) {
        match self {
            ast::Item::Entry(entry) => entry.eval(ctx),
            ast::Item::PrefixBlock(block) => block.eval(ctx),
            ast::Item::Directive(_) => {}
        }
    }
}

impl Eval for ast::Entry {
    type Output = ();
    fn eval(&self, ctx: &mut EvalContext) {
        let Some(key) = self.key() else {
            ctx.errors.push(ConfigError::MissingSyntax { range: self.syntax().text_range() });
            return;
        };
        let Some(value) = self.value() else {
            ctx.errors.push(ConfigError::MissingSyntax { range: self.syntax().text_range() });
            return;
        };

        let Some(key_text) = key.eval(ctx) else { return };
        let full_key =
            if ctx.prefix.is_empty() { key_text } else { format!("{}{key_text}", ctx.prefix) };
        let key_id = ctx.arena.alloc(&full_key);

        let Some(value_raw) = value.eval(ctx) else { return };
        ctx.entries.push((key_id, value_raw));
    }
}

impl Eval for ast::PrefixBlock {
    type Output = ();
    fn eval(&self, ctx: &mut EvalContext) {
        let Some(key) = self.key() else {
            ctx.errors.push(ConfigError::MissingSyntax { range: self.syntax().text_range() });
            return;
        };
        let Some(key_text) = key.eval(ctx) else { return };

        let prev_prefix_len = ctx.prefix.len();
        ctx.prefix.push_str(&key_text);

        for item in self.items() {
            item.eval(ctx);
        }

        ctx.prefix.truncate(prev_prefix_len);
    }
}

impl Eval for ast::Key {
    type Output = Option<String>;
    fn eval(&self, ctx: &mut EvalContext) -> Option<String> {
        match self {
            ast::Key::Word(word_key) => match word_key.word() {
                Some(token) => Some(token.text().to_string()),
                None => {
                    ctx.errors
                        .push(ConfigError::MissingSyntax { range: word_key.syntax().text_range() });
                    None
                }
            },
            ast::Key::Quoted(quoted_key) => match quoted_key.string() {
                Some(t) => {
                    let text = t.text();
                    if text.is_empty() || !text.contains('\\') {
                        Some(text.to_string())
                    } else {
                        resolve_escapes(text, t.text_range(), &mut ctx.errors)
                    }
                }
                None => {
                    ctx.errors.push(ConfigError::MissingSyntax {
                        range: quoted_key.syntax().text_range(),
                    });
                    None
                }
            },
        }
    }
}

impl Eval for ast::Value {
    type Output = Option<ValueRaw>;
    fn eval(&self, ctx: &mut EvalContext) -> Option<ValueRaw> {
        match self {
            ast::Value::Boolean(b) => {
                let kind = match b.kind() {
                    Some(k) => k,
                    None => {
                        ctx.errors
                            .push(ConfigError::MissingSyntax { range: b.syntax().text_range() });
                        return None;
                    }
                };
                let val = match kind {
                    BooleanKind::True => true,
                    BooleanKind::False => false,
                };
                Some(ValueRaw::Bool(val))
            }
            ast::Value::Integer(i) => {
                let text = i.eval(ctx)?;
                let id = ctx.arena.alloc(&text);
                Some(ValueRaw::Integer(id))
            }
            ast::Value::LineString(ls) => {
                let text = ls.string().map(|t| t.text().to_string()).unwrap_or_default();
                let id = ctx.arena.alloc(&text);
                Some(ValueRaw::String(id))
            }
            ast::Value::QuotedString(qs) => {
                let text = match qs.string() {
                    Some(t) => {
                        let raw = t.text();
                        if raw.is_empty() || !raw.contains('\\') {
                            raw.to_string()
                        } else {
                            resolve_escapes(raw, t.text_range(), &mut ctx.errors)?
                        }
                    }
                    None => String::new(),
                };
                let id = ctx.arena.alloc(&text);
                Some(ValueRaw::String(id))
            }
            ast::Value::BlockString(bs) => {
                let text = bs.eval(ctx);
                let id = ctx.arena.alloc(&text);
                Some(ValueRaw::String(id))
            }
        }
    }
}

impl Eval for ast::Integer {
    type Output = Option<String>;

    fn eval(&self, ctx: &mut EvalContext) -> Self::Output {
        let mut text = String::new();
        if let Some(sign) = self.sign() {
            text.push_str(sign.text());
        }
        match self.numeral() {
            Some(n) => text.push_str(n.text()),
            None => {
                ctx.errors.push(ConfigError::MissingSyntax { range: self.syntax().text_range() });
                return None;
            }
        }
        Some(text)
    }
}

impl Eval for ast::BlockString {
    type Output = String;

    fn eval(&self, _ctx: &mut EvalContext) -> Self::Output {
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

/// Resolve escape sequences in a raw string token.
///
/// Supported escapes:
///   `\"` → `"`   `\'` → `'`   `\\` → `\`
///   `\n` → LF    `\r` → CR    `\t` → TAB
fn resolve_escapes(
    text: &str,
    token_range: TextRange,
    errors: &mut Vec<ConfigError>,
) -> Option<String> {
    debug_assert!(text.contains('\\'));

    let base_offset: u32 = token_range.start().into();
    let mut result = String::with_capacity(text.len());
    let mut chars = text.char_indices();

    while let Some((_, c)) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        match chars.next() {
            Some((_, '"')) => result.push('"'),
            Some((_, '\'')) => result.push('\''),
            Some((_, '\\')) => result.push('\\'),
            Some((_, 'n')) => result.push('\n'),
            Some((_, 'r')) => result.push('\r'),
            Some((_, 't')) => result.push('\t'),
            Some((byte_offset, other)) => {
                let esc_start = base_offset + (byte_offset as u32 - 1);
                let esc_len = 1 + other.len_utf8() as u32;
                let range = TextRange::at(TextSize::from(esc_start), TextSize::from(esc_len));
                errors.push(ConfigError::InvalidEscape { range, sequence: format!("\\{other}") });
                return None;
            }
            None => {
                let esc_start = base_offset + (text.len() as u32 - 1);
                let range = TextRange::at(TextSize::from(esc_start), TextSize::from(1));
                errors.push(ConfigError::InvalidEscape { range, sequence: "\\".to_string() });
                return None;
            }
        }
    }

    Some(result)
}

pub(crate) fn eval(source_file: &ast::SourceFile) -> EvalContext {
    let mut ctx = EvalContext::new();
    source_file.eval(&mut ctx);
    ctx
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_basic() {
        let range = TextRange::default();
        let mut errors = Vec::new();

        assert_eq!(
            resolve_escapes(r#"hello \"world\""#, range, &mut errors),
            Some(r#"hello "world""#.into())
        );
        assert!(errors.is_empty());

        assert_eq!(resolve_escapes(r"a\\b", range, &mut errors), Some("a\\b".into()));
        assert!(errors.is_empty());

        assert_eq!(
            resolve_escapes(r"line1\nline2", range, &mut errors),
            Some("line1\nline2".into())
        );
        assert!(errors.is_empty());

        assert_eq!(resolve_escapes(r"\t\r", range, &mut errors), Some("\t\r".into()));
        assert!(errors.is_empty());

        assert_eq!(resolve_escapes(r"can\'t", range, &mut errors), Some("can't".into()));
        assert!(errors.is_empty());
    }

    #[test]
    fn escape_invalid() {
        let range = TextRange::default();
        let mut errors = Vec::new();

        assert_eq!(resolve_escapes(r"\x", range, &mut errors), None);
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(&errors[0], ConfigError::InvalidEscape { sequence, .. } if sequence == r"\x")
        );
    }

    #[test]
    fn escape_trailing_backslash() {
        let range = TextRange::default();
        let mut errors = Vec::new();

        assert_eq!(resolve_escapes("trail\\", range, &mut errors), None);
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(&errors[0], ConfigError::InvalidEscape { sequence, .. } if sequence == r"\")
        );
    }

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
