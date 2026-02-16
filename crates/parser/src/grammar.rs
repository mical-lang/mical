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
