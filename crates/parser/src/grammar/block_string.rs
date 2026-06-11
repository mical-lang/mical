use super::emit_trailing_trivia;
use crate::parser::{Marker, Parser};
use mical_cli_lexer::{Chomp, LineHead, Style};
use mical_cli_syntax::{SyntaxKind, T};

pub(super) fn block_string(
    p: &mut Parser,
    entry_m: Marker,
    style: Style,
    indent_indicator: Option<u8>,
    chomp: Option<Chomp>,
) {
    let bs = p.start();
    // The body is measured against the entry line's indent, so it must be
    // captured before the header consumes the line.
    let i_parent = p.indent();
    header(p, style, indent_indicator.is_some(), chomp);

    let i_base = match indent_indicator {
        Some(n) => Some(i_parent + n as u32),
        None => infer_base_indent(p),
    };
    // A base indent at or above the parent level means the line after the
    // header already belongs to the outer scope: the body is empty.
    if let Some(i_base) = i_base.filter(|&i_base| i_base > i_parent) {
        body_lines(p, i_parent, i_base);
    }

    bs.complete(p, SyntaxKind::BLOCK_STRING);
    entry_m.complete(p, SyntaxKind::ENTRY);
}

fn header(p: &mut Parser, style: Style, has_indent_indicator: bool, chomp: Option<Chomp>) {
    let m = p.start();
    let style_kind = match style {
        Style::Literal => T![|],
        Style::Folded => T![>],
    };
    p.token(style_kind, 1);
    if has_indent_indicator {
        p.token(T![numeral], 1);
    }
    if let Some(chomp) = chomp {
        let chomp_kind = match chomp {
            Chomp::Keep => T![+],
            Chomp::Strip => T![-],
        };
        p.token(chomp_kind, 1);
    }
    emit_trailing_trivia(p);
    p.finish_line();
    m.complete(p, SyntaxKind::BLOCK_STRING_HEADER);
}

fn infer_base_indent(p: &mut Parser) -> Option<u32> {
    if p.at_eof() {
        return None;
    }
    if p.rest_len() == 0 {
        p.error("cannot infer base indent from an empty line", 0);
        return None;
    }
    // A whitespace-only line counts fully: all of its spaces are its indent.
    Some(p.indent())
}

fn body_lines(p: &mut Parser, i_parent: u32, i_base: u32) {
    while !p.at_eof() {
        // `indent` counts leading spaces only, so a tab is literal content on
        // a content line and ends the indentation otherwise — exactly the
        // spec's tab-neutral classification.
        if p.head() == LineHead::Blank {
            if p.rest_len() > 0 {
                p.token(T![' '], p.rest_len());
            }
            let lm = p.start();
            lm.complete(p, SyntaxKind::LINE_STRING);
            p.finish_line();
        } else if p.indent() >= i_base {
            p.token(T![' '], i_base);
            let lm = p.start();
            p.token(T![string], p.rest_len());
            lm.complete(p, SyntaxKind::LINE_STRING);
            p.finish_line();
        } else if p.indent() <= i_parent {
            break;
        } else {
            p.error("block string line has insufficient indentation", p.indent());
            p.token(T![' '], p.indent());
            let em = p.start();
            p.token(T![string], p.rest_len());
            em.complete(p, SyntaxKind::ERROR);
            p.finish_line();
        }
    }
}
