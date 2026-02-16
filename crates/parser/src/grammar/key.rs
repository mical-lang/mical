use super::*;

pub(super) const KEY_FIRST: TokenSet = TokenSet::new([
    T![word],
    T![numeral],
    T![true],
    T![false],
    T![-],
    T![+],
    T![|],
    T![>],
    T!['"'],
    T!['\''],
    T!['{'],
    T!['}'],
]);

/// `T![' ']`, `T!['\n']`, `T!['\t']`, or EOF
pub(super) const KEY_LAST: TokenSet = TokenSet::new([T![' '], T!['\n'], T!['\t']]);

pub(super) fn key(p: &mut Parser) -> CompletedMarker {
    assert!(p.at_ts(KEY_FIRST));

    match unsafe { p.current().unwrap_unchecked() } {
        quote @ (T!['"'] | T!['\'']) => quoted_key(p, quote),
        _ => word_key(p),
    }
}

fn word_key(p: &mut Parser) -> CompletedMarker {
    let m = p.start();

    let mut count = 0;
    while !(p.nth_at_ts(count, KEY_LAST) || p.nth_at_eof(count)) {
        count += 1;
    }
    p.bump_remap(T![word], count);

    m.complete(p, WORD_KEY)
}

fn quoted_key(p: &mut Parser, quote: SyntaxKind) -> CompletedMarker {
    assert!((p.at(T!['"']) || p.at(T!['\''])) && p.at(quote));

    let m = p.start();

    p.bump(quote);

    p.bump(T![string]);

    if p.eat(quote) {
        p.error("missing closing quote");
    }

    if !(p.at_ts(KEY_LAST) || p.at_eof()) {
        p.error("unexpected token after quoted key");
        let m = p.start();
        while !(p.at_ts(KEY_LAST) || p.at_eof()) {
            p.bump_any();
        }
        m.complete(p, ERROR);
    }

    m.complete(p, QUOTED_KEY)
}
