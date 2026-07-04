use super::emit_quoted;
use crate::parser::Parser;
use mical_cli_syntax::{SyntaxKind, T};

pub(super) struct ParsedKey {
    pub(super) unclosed_quote: bool,
}

pub(super) fn parse_key(p: &mut Parser) -> ParsedKey {
    let Some(quoted) = p.quoted() else {
        let m = p.start();
        let word = p.word().expect("an item line guarantees a key word");
        p.token(T![word], word.text().len());
        m.complete(p, SyntaxKind::WORD_KEY);
        return ParsedKey { unclosed_quote: false };
    };

    let m = p.start();
    emit_quoted(p, quoted);
    let junk_len = if quoted.is_closed() { p.word().map_or(0, |w| w.text().len()) } else { 0 };
    if junk_len > 0 {
        p.error("unexpected token after quoted key", junk_len);
        let em = p.start();
        p.token(T![string], junk_len);
        em.complete(p, SyntaxKind::ERROR);
    }
    m.complete(p, SyntaxKind::QUOTED_KEY);
    ParsedKey { unclosed_quote: !quoted.is_closed() }
}
