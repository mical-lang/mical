use super::{block_string, emit_quoted, emit_trailing_trivia, items, peek_item_line};
use crate::parser::{Marker, Parser};
use mical_cli_lexer::{Quoted, Sign, ValueKind};
use mical_cli_syntax::{SyntaxKind, T};

pub(super) fn parse_value(p: &mut Parser, m: Marker, suppress_missing_value: bool) {
    // A leading quote protects `#` from comment recognition, so quoted values
    // must be recognized before splitting off the trailing trivia.
    if let Some(quoted) = p.quoted() {
        quoted_value(p, m, quoted);
        return;
    }

    let content = p.trailing().0;
    if content.is_empty() {
        key_alone(p, m, suppress_missing_value);
        return;
    }

    match p.classify_value(content) {
        ValueKind::BlockHeader { style, indent, chomp } => {
            block_string::block_string(p, m, style, indent, chomp);
            return;
        }
        ValueKind::Boolean { value } => {
            let vm = p.start();
            let kind = if value { T![true] } else { T![false] };
            p.token(kind, content.len());
            vm.complete(p, SyntaxKind::BOOLEAN);
        }
        ValueKind::Integer { sign, radix: _ } => {
            let vm = p.start();
            let sign_len = sign.is_some() as usize;
            if let Some(sign) = sign {
                let kind = match sign {
                    Sign::Plus => T![+],
                    Sign::Minus => T![-],
                };
                p.token(kind, 1);
            }
            p.token(T![numeral], content.len() - sign_len);
            vm.complete(p, SyntaxKind::INTEGER);
        }
        ValueKind::LineString => {
            let vm = p.start();
            p.token(T![string], content.len());
            vm.complete(p, SyntaxKind::LINE_STRING);
        }
    }
    emit_trailing_trivia(p);
    p.finish_line();
    m.complete(p, SyntaxKind::ENTRY);
}

fn quoted_value(p: &mut Parser, m: Marker, quoted: Quoted<'_>) {
    let vm = p.start();
    emit_quoted(p, quoted);
    vm.complete(p, SyntaxKind::QUOTED_STRING);

    let (junk_and_spaces, _) = p.trailing();
    if !junk_and_spaces.is_empty() {
        let spaces_len = p.separator().map_or(0, |s| s.spaces().len());
        if spaces_len > 0 {
            p.token(T![' '], spaces_len);
        }
        let junk_len = junk_and_spaces.len() - spaces_len;
        p.error("unexpected token after value", junk_len);
        let em = p.start();
        p.token(T![string], junk_len);
        em.complete(p, SyntaxKind::ERROR);
    }
    emit_trailing_trivia(p);
    p.finish_line();
    m.complete(p, SyntaxKind::ENTRY);
}

fn key_alone(p: &mut Parser, m: Marker, suppress_missing_value: bool) {
    match peek_item_line(p, 1) {
        Some((_, next_indent)) if next_indent > p.indent() => {
            emit_trailing_trivia(p);
            p.finish_line();
            // Trivia between the opener and the first body item is consumed
            // by the body's own items() lookahead.
            items(p, next_indent);
            m.complete(p, SyntaxKind::PREFIX_BLOCK);
        }
        _ => {
            if !suppress_missing_value {
                p.error("missing value for the key", 0);
            }
            emit_trailing_trivia(p);
            p.finish_line();
            m.complete(p, SyntaxKind::ENTRY);
        }
    }
}
