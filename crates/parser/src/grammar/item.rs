use super::*;

pub(super) fn item(p: &mut Parser) {
    while p.eat(T!['\n']) {}
    if p.at_eof() {
        return;
    }

    if p.at(T![#]) {
        if p.nth_at(1, T![word]) {
            directive(p);
        } else {
            comment(p);
        }
        return;
    }

    // leading spaces (indent)
    let indent_level;
    if p.at(T![' ']) {
        indent_level = unsafe { p.current_len().unwrap_unchecked() };
        p.bump(T![' ']);
    } else {
        indent_level = 0;
    }
    if p.at(T!['\t']) {
        p.error("tab indent is not allowed, skipping this line");
        let m = p.start();
        skip_to_end_of_line(p);
        m.complete(p, ERROR);
        return;
    }

    if p.at(T![#]) {
        comment(p);
        return;
    }

    entry_or_prefix_block(p, indent_level);
}

fn directive(p: &mut Parser) {
    assert!(p.at(T![#]) && p.nth_at(1, T![word]));

    let m = p.start();

    p.bump(T![#]);
    p.bump(T![word]);
    value::line_string(p);

    m.complete(p, DIRECTIVE);
}

fn comment(p: &mut Parser) {
    assert!(p.at(T![#]));

    let m = p.start();

    while let Some(current) = p.current() {
        if current == T!['\n'] {
            break;
        }
        p.bump_any();
    }

    m.complete(p, COMMENT);
}

fn entry_or_prefix_block(p: &mut Parser, indent_level: u32) {
    assert!(!(p.at(T!['\n']) || p.at_eof()));

    // key
    if !p.at_ts(key::KEY_FIRST) {
        p.error("expected a key");
        let m = p.start();
        skip_to_end_of_line(p);
        m.complete(p, ERROR);
        return;
    }
    let m = p.start();
    key::key(p);

    // error (missing value)
    assert!(p.at_ts(key::KEY_LAST) || p.at_eof());
    if p.at(T!['\n']) || p.at_eof() {
        p.error("missing value for the key");
        m.complete(p, ENTRY);
        return;
    }
    assert!(p.at(T![' ']) || p.at(T!['\t']));

    // separator
    p.eat(T![' ']);
    if p.at(T!['\t']) {
        p.error("tab separating is not allowed");
        let m = p.start();
        p.bump(T!['\t']);
        m.complete(p, ERROR);
    }

    if p.at(T!['{']) && p.nth_at(1, T!['\n']) {
        prefix_block(p, m);
    } else {
        entry(p, m, indent_level);
    }
}

fn prefix_block(p: &mut Parser, m: Marker) {
    assert!(p.at(T!['{']) && p.nth_at(1, T!['\n']));

    p.bump(T!['{']);
    p.bump(T!['\n']);

    loop {
        if p.at_eof() {
            p.error("missing closing '}' for prefix block");
            break;
        }
        if p.at(T!['}']) && (p.nth_at(1, T!['\n']) || p.at_eof()) {
            p.bump(T!['}']);
            p.eat(T!['\n']);
            break;
        }
        item(p);
    }

    m.complete(p, PREFIX_BLOCK);
}

fn entry(p: &mut Parser, m: Marker, indent_level: u32) {
    assert!(!(p.at(T!['\n']) || p.at_eof()));

    // value
    if p.at_ts(value::VALUE_FIRST) {
        value::value(p, indent_level);
    }

    // trailing spaces
    p.eat(T![' ']);

    if !(p.at(T!['\n']) || p.at_eof()) {
        p.error("unexpected token after value");
        let m = p.start();
        skip_to_end_of_line(p);
        m.complete(p, ERROR);
    }

    m.complete(p, ENTRY);
}

fn skip_to_end_of_line(p: &mut Parser) {
    while !(p.at(T!['\n']) || p.at_eof()) {
        p.bump_any();
    }
}
