use crate::parser::Parser;
use mical_cli_lexer::{Line, LineHead, Quote, QuotedToken};
use mical_cli_syntax::{SyntaxKind, T};

mod block_string;
mod item;
mod key;
mod value;

pub(crate) fn source_file(p: &mut Parser) {
    let m = p.start();
    items(p, 0);
    while !p.at_eof() {
        emit_trivia_line(p);
    }
    m.complete(p, SyntaxKind::SOURCE_FILE);
}

fn items(p: &mut Parser, body_indent: u32) {
    loop {
        let Some((trivia_count, indent)) = peek_item_line(p, 0) else { return };
        // Trivia lines before a dedent are left unconsumed: they bind to the
        // *following* item's level, not to the block being closed.
        if indent < body_indent {
            return;
        }
        for _ in 0..trivia_count {
            emit_trivia_line(p);
        }
        if indent > body_indent {
            let message = if indent > p.last_item_indent {
                "unexpected indentation"
            } else {
                "indentation does not match any enclosing block"
            };
            p.error(message, indent);
            // Recover by parsing the line as an item at this level.
        }
        p.last_item_indent = indent;
        item::item(p);
    }
}

fn peek_item_line(p: &Parser, from_nth: usize) -> Option<(usize, u32)> {
    let mut n = from_nth;
    loop {
        let line = p.nth_line(n)?;
        if !is_trivia_line(line) {
            return Some((n - from_nth, line.indent()));
        }
        n += 1;
    }
}

fn is_trivia_line(line: Line) -> bool {
    match line.head() {
        LineHead::Blank => true,
        LineHead::Hash => !line.is_directive(),
        LineHead::Tab | LineHead::Other => false,
    }
}

fn emit_trivia_line(p: &mut Parser) {
    match p.head() {
        LineHead::Blank => {
            if p.rest_len() > 0 {
                p.token(T![' '], p.rest_len());
            }
        }
        // The COMMENT token covers the indentation too: the spec removes a
        // comment together with the whitespace preceding it.
        LineHead::Hash => p.token(T![comment], p.rest_len()),
        LineHead::Tab | LineHead::Other => unreachable!("not a trivia line"),
    }
    p.finish_line();
}

fn emit_trailing_trivia(p: &mut Parser) {
    let split = p.split_comment();
    debug_assert_eq!(split.value_len, 0, "value text must be consumed before trailing trivia");
    if split.trailing_space_len > 0 {
        p.token(T![' '], split.trailing_space_len);
    }
    if split.comment_len > 0 {
        p.token(T![comment], split.comment_len);
    }
}

fn emit_quoted(p: &mut Parser, quoted: QuotedToken) {
    let quote_kind = match quoted.quote {
        Quote::Single => T!['\''],
        Quote::Double => T!['"'],
    };
    if !quoted.closed {
        p.error("missing closing quote", quoted.consumed_len());
    }
    p.token(quote_kind, 1);
    if quoted.content_len > 0 {
        p.token(T![string], quoted.content_len);
    }
    if quoted.closed {
        p.token(quote_kind, 1);
    }
}
