use super::*;

pub(super) const VALUE_FIRST: TokenSet = TokenSet::new([
    T![word],
    T![numeral],
    T![true],
    T![false],
    T![-],
    T![+],
    T![|],
    T![>],
    T![#],
    T!['"'],
    T!['\''],
    T!['{'],
    T!['}'],
]);

pub(super) fn value(p: &mut Parser, indent_level: u32) {
    assert!(p.at_ts(VALUE_FIRST));

    match unsafe { p.current().unwrap_unchecked() } {
        quote @ (T!['"'] | T!['\'']) => quoted_value(p, quote),
        T![|] | T![>] => {
            let mut shift = 1;
            if p.nth_at(shift, T![+]) || p.nth_at(shift, T![-]) {
                shift += 1;
            }
            if is_rest_of_line_blank(p, shift) {
                block_string(p, indent_level);
            } else {
                line_string(p)
            }
        }
        T![true] | T![false] if is_rest_of_line_blank(p, 1) => {
            boolean(p);
        }
        T![numeral] if is_rest_of_line_blank(p, 1) => {
            integer(p);
        }
        T![+] | T![-] if p.nth_at(1, T![numeral]) && is_rest_of_line_blank(p, 2) => {
            integer(p);
        }
        _ => line_string(p),
    }
}

fn boolean(p: &mut Parser) {
    assert!(p.at(T![true]) || p.at(T![false]));

    let m = p.start();
    p.bump_any(); // true or false
    m.complete(p, BOOLEAN);
}

fn integer(p: &mut Parser) {
    assert!(p.at(T![+]) || p.at(T![-]) || p.at(T![numeral]));

    let m = p.start();
    if p.at(T![+]) || p.at(T![-]) {
        p.bump_any(); // sign
    }
    p.bump(T![numeral]);
    m.complete(p, INTEGER);
}

pub(super) fn line_string(p: &mut Parser) {
    let m = p.start();

    let mut count = 0;
    while !(p.nth_at(count, T!['\n']) || p.nth_at_eof(count)) {
        count += 1;
    }
    p.bump_remap(T![string], count);

    m.complete(p, LINE_STRING);
}

fn quoted_value(p: &mut Parser, quote: SyntaxKind) {
    assert!((quote == T!['"'] || quote == T!['\'']) && p.at(quote));

    let m = p.start();

    p.bump(quote);

    p.bump(T![string]);

    if !p.eat(quote) {
        p.error("missing closing quote");
    }

    m.complete(p, QUOTED_STRING);
}

fn block_string(p: &mut Parser, indent_level: u32) {
    assert!(p.at(T![|]) || p.at(T![>]));

    let m = p.start();

    block_string_header(p);

    // Scan ahead to find base_indent: the indent of the first line with content.
    // Whitespace-only lines and empty lines are skipped during this scan.
    let base_indent = {
        let mut offset = 0;
        loop {
            if p.nth_at_eof(offset) {
                break None;
            }
            if p.nth_at(offset, T![' ']) {
                let indent = unsafe { p.nth_len(offset).unwrap_unchecked() };
                offset += 1;
                if p.nth_at(offset, T!['\n']) || p.nth_at_eof(offset) {
                    // Whitespace-only line, skip
                    offset += 1;
                    continue;
                }
                break Some(indent); // Found a line with content
            } else if p.nth_at(offset, T!['\n']) {
                offset += 1;
                continue;
            } else {
                break Some(0); // Content at column 0.
            }
        }
    };

    let Some(base_indent) = base_indent else {
        m.complete(p, BLOCK_STRING);
        return;
    };

    // `base_indent` must be deeper than `indent_level`. Otherwise the content belongs to the outer
    // scope, not this block.
    if base_indent <= indent_level {
        m.complete(p, BLOCK_STRING);
        return;
    }

    // Parse each line using `base_indent` as the reference.
    //
    // For a line with `line_indent` spaces:
    //   line_indent >= base_indent  → content line (strip base_indent spaces)
    //   line_indent <= indent_level → block ends
    //   otherwise (between the two) →
    //     whitespace-only → empty line (allowed)
    //     with content    → error (insufficient indentation)
    while let Some(current) = p.current() {
        match current {
            T!['\n'] => {
                // Completely empty line (no spaces at all).
                block_string_empty_line(p);
            }
            T![' '] => {
                let line_indent = unsafe { p.current_len().unwrap_unchecked() };

                if line_indent >= base_indent {
                    p.bump_upto(T![' '], base_indent);

                    // After stripping base_indent, check if the rest is still blank.
                    // e.g. a line of only spaces deeper than base_indent.
                    let is_blank = if p.at(T![' ']) {
                        p.nth_at(1, T!['\n']) || p.nth_at_eof(1)
                    } else {
                        p.at(T!['\n']) || p.at_eof()
                    };

                    if is_blank {
                        p.eat(T![' ']);
                        block_string_empty_line(p);
                    } else {
                        block_string_content_line(p);
                    }
                } else if line_indent <= indent_level {
                    break;
                } else {
                    // indent_level < line_indent < base_indent
                    if p.nth_at(1, T!['\n']) || p.nth_at_eof(1) {
                        // Whitespace-only lines in this range are allowed.
                        p.bump(T![' ']);
                        block_string_empty_line(p);
                    } else {
                        // Content in this range is an error.
                        p.error("block string line has insufficient indentation");
                        let m = p.start();
                        p.bump(T![' ']);
                        while !(p.at(T!['\n']) || p.at_eof()) {
                            p.bump_any();
                        }
                        m.complete(p, ERROR);
                    }
                }
            }
            _ => {
                // Non-space content at column 0.
                break;
            }
        }

        // Consume inter-line newline separator if the block continues.
        // If it doesn't continue, leave the newline for the caller just like other value types.
        if !p.at(T!['\n']) || !block_continues_after_newline(p, indent_level) {
            break;
        }
        p.bump(T!['\n']);
    }

    m.complete(p, BLOCK_STRING);
}

fn block_continues_after_newline(p: &Parser, indent_level: u32) -> bool {
    assert!(p.at(T!['\n']));

    if p.nth_at_eof(1) {
        return false;
    }
    if p.nth_at(1, T!['\n']) {
        return true; // Empty line → block continues
    }
    if p.nth_at(1, T![' ']) {
        let indent = unsafe { p.nth_len(1).unwrap_unchecked() };
        // indent > indent_level covers both valid content and error lines;
        // all are still processed within the block.
        return indent > indent_level;
    }
    // Non-space at column 0 → block ends
    false
}

fn block_string_empty_line(p: &mut Parser) {
    assert!(p.at(T!['\n']) || p.at_eof());

    let m = p.start();
    m.complete(p, LINE_STRING);
}

fn block_string_content_line(p: &mut Parser) {
    assert!(!(p.at(T!['\n']) || p.at_eof()));

    line_string(p);
}

fn block_string_header(p: &mut Parser) {
    assert!(p.at(T![|]) || p.at(T![>]));

    let m = p.start();

    p.bump_any(); // | or >

    if p.at(T![+]) {
        p.bump(T![+]);
    } else if p.at(T![-]) {
        p.bump(T![-]);
    }

    p.eat(T![' ']);
    p.eat(T!['\n']);

    m.complete(p, BLOCK_STRING_HEADER);
}
