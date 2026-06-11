use super::{block_string, emit_quoted, emit_trailing_trivia, items, peek_item_line};
use crate::parser::{Marker, Parser};
use mical_cli_lexer::{QuotedToken, Sign, ValueKind};
use mical_cli_syntax::{SyntaxKind, T};

pub(super) fn parse_value(p: &mut Parser, m: Marker, suppress_missing_value: bool) {
    // A leading quote protects `#` from comment recognition, so quoted values
    // must be scanned before comment splitting.
    if let Some(quoted) = p.scan_quoted() {
        quoted_value(p, m, quoted);
        return;
    }

    let value_len = p.split_comment().value_len;
    if value_len == 0 {
        key_alone(p, m, suppress_missing_value);
        return;
    }

    match p.classify_value(value_len) {
        ValueKind::BlockHeader { style, indent, chomp } => {
            block_string::block_string(p, m, style, indent, chomp);
            return;
        }
        ValueKind::Boolean { value } => {
            let vm = p.start();
            let kind = if value { T![true] } else { T![false] };
            p.token(kind, value_len);
            vm.complete(p, SyntaxKind::BOOLEAN);
        }
        ValueKind::Integer { sign } => {
            let vm = p.start();
            let sign_len = sign.is_some() as u32;
            if let Some(sign) = sign {
                let kind = match sign {
                    Sign::Plus => T![+],
                    Sign::Minus => T![-],
                };
                p.token(kind, 1);
            }
            p.token(T![numeral], value_len - sign_len);
            vm.complete(p, SyntaxKind::INTEGER);
        }
        ValueKind::LineString => {
            let vm = p.start();
            p.token(T![string], value_len);
            vm.complete(p, SyntaxKind::LINE_STRING);
        }
    }
    emit_trailing_trivia(p);
    p.finish_line();
    m.complete(p, SyntaxKind::ENTRY);
}

fn quoted_value(p: &mut Parser, m: Marker, quoted: QuotedToken) {
    let vm = p.start();
    emit_quoted(p, quoted);
    vm.complete(p, SyntaxKind::QUOTED_STRING);

    let junk_and_spaces_len = p.split_comment().value_len;
    if junk_and_spaces_len > 0 {
        let leading_space_len = p.scan_separator().space_len;
        if leading_space_len > 0 {
            p.token(T![' '], leading_space_len);
        }
        let junk_len = junk_and_spaces_len - leading_space_len;
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
