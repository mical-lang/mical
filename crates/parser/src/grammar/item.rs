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
        LineHead::Other => entry_or_prefix_block(p),
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
    p.token(T![word], p.scan_word());
    let space_len = p.scan_separator().space_len;
    if space_len > 0 {
        p.token(T![' '], space_len);
    }
    let args_len = p.split_comment().value_len;
    if args_len > 0 {
        let args = p.start();
        p.token(T![string], args_len);
        args.complete(p, SyntaxKind::LINE_STRING);
    }
    emit_trailing_trivia(p);
    p.finish_line();
    m.complete(p, SyntaxKind::DIRECTIVE);
}

fn entry_or_prefix_block(p: &mut Parser) {
    let m = p.start();
    let parsed_key = key::parse_key(p);
    let separator = p.scan_separator();
    if separator.space_len > 0 {
        p.token(T![' '], separator.space_len);
    }
    if separator.tab_run_len > 0 {
        p.error("tab separating is not allowed", separator.tab_run_len);
        let em = p.start();
        emit_whitespace_runs(p, separator.tab_run_len);
        em.complete(p, SyntaxKind::ERROR);
    }
    value::parse_value(p, m, parsed_key.unclosed_quote);
}

fn emit_whitespace_runs(p: &mut Parser, mut len: u32) {
    while len > 0 {
        let run = p.scan_whitespace_run().expect("whitespace run within separator");
        debug_assert!(run.len <= len);
        p.token(if run.is_tab { T!['\t'] } else { T![' '] }, run.len);
        len -= run.len;
    }
}
