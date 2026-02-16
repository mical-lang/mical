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
            if p.nth_at(shift, T!['\n']) {
                block_string(p, indent_level);
            } else {
                line_string(p)
            }
        }
        _ => line_string(p),
    }
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
    loop {
        if p.at_eof() {
            break;
        }

        // Completely empty line (no spaces at all).
        if p.at(T!['\n']) {
            block_string_empty_line(p);
            continue;
        }

        if p.at(T![' ']) {
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
                continue;
            }

            if line_indent <= indent_level {
                break;
            }

            // indent_level < line_indent < base_indent
            // Whitespace-only lines in this range are allowed.
            if p.nth_at(1, T!['\n']) || p.nth_at_eof(1) {
                p.bump(T![' ']);
                block_string_empty_line(p);
                continue;
            }

            // Content in this range is an error.
            p.error("block string line has insufficient indentation");
            let m = p.start();
            p.bump(T![' ']);
            while !(p.at(T!['\n']) || p.at_eof()) {
                p.bump_any();
            }
            m.complete(p, ERROR);
            p.eat(T!['\n']);
            continue;
        }

        // Non-space content at column 0.
        // Since base_indent > indent_level >= 0, column 0 ends the block.
        break;
    }

    m.complete(p, BLOCK_STRING);
}

fn block_string_empty_line(p: &mut Parser) {
    assert!(p.at(T!['\n']) || p.at_eof());

    let m = p.start();
    m.complete(p, LINE_STRING);

    p.eat(T!['\n']);
}

fn block_string_content_line(p: &mut Parser) {
    assert!(!(p.at(T!['\n']) || p.at_eof()));

    line_string(p);

    p.eat(T!['\n']);
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

    p.bump(T!['\n']);

    m.complete(p, BLOCK_STRING_HEADER);
}
