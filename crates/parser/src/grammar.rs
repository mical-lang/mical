use crate::{parser::*, token_set::TokenSet};
use mical_syntax::{SyntaxKind, SyntaxKind::*, T};

mod item;
mod key;
mod value;

pub(crate) fn source_file(p: &mut Parser) {
    let m = p.start();

    p.eat(T![shebang]);

    while !p.at_eof() {
        item::item(p);
    }

    m.complete(p, SOURCE_FILE);
}

fn is_rest_of_line_blank(p: &Parser, n: usize) -> bool {
    if p.nth_at(n, T!['\n']) || p.nth_at_eof(n) {
        return true;
    }
    p.nth_at(n, T![' ']) && (p.nth_at(n + 1, T!['\n']) || p.nth_at_eof(n + 1))
}
