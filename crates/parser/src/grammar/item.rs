use super::{emit_trailing_trivia, key, value};
use crate::parser::Parser;
use mical_cli_lexer::LineHead;
use mical_cli_syntax::{SyntaxKind, T};

pub(super) fn item(p: &mut Parser) {
    // An item's indentation stays outside its node (CST shape rule).
    if p.indent() > 0 {
        p.token(T![' '], p.indent());
    }
    match p.head() {
        LineHead::Tab => tab_indent_error_line(p),
        LineHead::Hash => directive(p),
        LineHead::Text => entry_or_prefix_block(p),
        LineHead::Blank => unreachable!("blank lines are trivia"),
    }
}

fn tab_indent_error_line(p: &mut Parser) {
    p.error("tab is not allowed in indentation", 1);
    let m = p.start();
    p.token(T![string], p.rest_len());
    p.finish_line();
    m.complete(p, SyntaxKind::ERROR);
}

fn directive(p: &mut Parser) {
    debug_assert!(p.is_directive());
    let m = p.start();
    p.token(T![#], 1);
    let name = p.word().expect("a directive head guarantees a name word");
    p.token(T![word], name.text().len());
    let spaces_len = p.separator().map_or(0, |s| s.spaces().len());
    if spaces_len > 0 {
        p.token(T![' '], spaces_len);
    }
    let args = p.trailing().0;
    if !args.is_empty() {
        let a = p.start();
        p.token(T![string], args.len());
        a.complete(p, SyntaxKind::LINE_STRING);
    }
    emit_trailing_trivia(p);
    p.finish_line();
    m.complete(p, SyntaxKind::DIRECTIVE);
}

fn entry_or_prefix_block(p: &mut Parser) {
    let m = p.start();
    let parsed_key = key::parse_key(p);
    if let Some(separator) = p.separator() {
        if !separator.spaces().is_empty() {
            p.token(T![' '], separator.spaces().len());
        }
        if let Some(tab_run) = separator.tab_run() {
            p.error("tab separating is not allowed", tab_run.len());
            let em = p.start();
            emit_whitespace_runs(p, tab_run);
            em.complete(p, SyntaxKind::ERROR);
        }
    }
    value::parse_value(p, m, parsed_key.unclosed_quote);
}

// A disallowed tab-run is preserved byte-for-byte by splitting it into
// homogeneous space/tab tokens, since the CST has no mixed-whitespace kind.
fn emit_whitespace_runs(p: &mut Parser, whitespace: &str) {
    let mut rest = whitespace;
    while let Some(&first) = rest.as_bytes().first() {
        let run_len = rest.bytes().take_while(|&b| b == first).count();
        p.token(if first == b'\t' { T!['\t'] } else { T![' '] }, run_len);
        rest = &rest[run_len..];
    }
}
