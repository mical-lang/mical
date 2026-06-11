mod macros;

use mical_cli_lexer::{Chomp::*, Quote::*, Sign::*, Style::*, scan_lines};

#[test]
fn scan_word_runs_until_whitespace_or_end() {
    assert_word!("hello world", 5);
    assert_word!("server.port 8080", 11);
    assert_word!("a#b value", 3);
    assert_word!("can't value", 5);
    assert_word!("42 value", 2);
    assert_word!("true value", 4);
    assert_word!("key\tvalue", 3);
    assert_word!("alone", 5);
    assert_word!("", 0);
}

#[test]
fn scan_quoted_terminated() {
    assert_quoted!("\"double\" v", { quote: Double, content_len: 6, closed: true });
    assert_quoted!("'single' v", { quote: Single, content_len: 6, closed: true });
    assert_quoted!("\"with space\" v", { quote: Double, content_len: 10, closed: true });
    assert_quoted!("\"\" v", { quote: Double, content_len: 0, closed: true });
    assert_quoted!("'' v", { quote: Single, content_len: 0, closed: true });
    assert_quoted!("\"quoted\"ppp value", { quote: Double, content_len: 6, closed: true });
}

#[test]
fn scan_quoted_unterminated() {
    assert_quoted!("\"unterminated value", { quote: Double, content_len: 18, closed: false });
    assert_quoted!("\"", { quote: Double, content_len: 0, closed: false });
}

#[test]
fn scan_quoted_non_quote_start_is_none() {
    assert_quoted!("plain", None);
    assert_quoted!("", None);
}

#[test]
fn scan_quoted_escapes_protect_matching_quote_only() {
    assert_quoted!("\"a\\\"b\" v", { quote: Double, content_len: 4, closed: true });
    assert_quoted!("'a\\'b' v", { quote: Single, content_len: 4, closed: true });
    assert_quoted!("\"a\\\\\" v", { quote: Double, content_len: 3, closed: true });
    // `\"` inside single quotes is not an escape that matters for closing.
    assert_quoted!("'a\\\"b'", { quote: Single, content_len: 4, closed: true });
    // A trailing lone backslash cannot escape past the end.
    assert_quoted!("\"a\\", { quote: Double, content_len: 2, closed: false });
}

#[test]
fn scan_separator_spaces_and_tab_runs() {
    assert_separator!(" value", space: 1, tab_run: 0);
    assert_separator!("   value", space: 3, tab_run: 0);
    assert_separator!("\tvalue", space: 0, tab_run: 1);
    assert_separator!(" \t value", space: 1, tab_run: 2);
    assert_separator!("\t\t value", space: 0, tab_run: 3);
    assert_separator!("value", space: 0, tab_run: 0);
    assert_separator!("", space: 0, tab_run: 0);
}

#[test]
fn split_comment_positional_hash() {
    assert_split_comment!("value", value: 5, space: 0, comment: 0);
    assert_split_comment!("value   ", value: 5, space: 3, comment: 0);
    assert_split_comment!("value # c", value: 5, space: 0, comment: 4);
    assert_split_comment!("value   # c", value: 5, space: 0, comment: 6);
    assert_split_comment!("# full line", value: 0, space: 0, comment: 11);
    assert_split_comment!("hello#world", value: 11, space: 0, comment: 0);
    assert_split_comment!("hello #world", value: 5, space: 0, comment: 7);
    // A tab does not start a comment.
    assert_split_comment!("x\t#y", value: 4, space: 0, comment: 0);
    // Mid-line quotes do not protect '#'.
    assert_split_comment!("a \"x # y\" tail", value: 4, space: 0, comment: 10);
    assert_split_comment!("", value: 0, space: 0, comment: 0);
}

#[test]
fn classify_value_block_headers() {
    assert_value!("|", BlockHeader { style: Literal, indent: None, chomp: None });
    assert_value!(">", BlockHeader { style: Folded, indent: None, chomp: None });
    assert_value!("|2", BlockHeader { style: Literal, indent: Some(2), chomp: None });
    assert_value!("|+", BlockHeader { style: Literal, indent: None, chomp: Some(Keep) });
    assert_value!("|-", BlockHeader { style: Literal, indent: None, chomp: Some(Strip) });
    assert_value!("|9-", BlockHeader { style: Literal, indent: Some(9), chomp: Some(Strip) });
    assert_value!(">1+", BlockHeader { style: Folded, indent: Some(1), chomp: Some(Keep) });
    // Out-of-order or trailing junk falls through to Line String.
    assert_value!("|-2", LineString);
    assert_value!("|abc", LineString);
    assert_value!("|not block", LineString);
    assert_value!("|+not block", LineString);
    assert_value!("|0", LineString);
}

#[test]
fn classify_value_booleans() {
    assert_value!("true", Boolean { value: true });
    assert_value!("false", Boolean { value: false });
    assert_value!("trueish", LineString);
    assert_value!("falsehood", LineString);
    assert_value!("true value", LineString);
}

#[test]
fn classify_value_integers() {
    assert_value!("0", Integer { sign: None });
    assert_value!("42", Integer { sign: None });
    assert_value!("1_000", Integer { sign: None });
    assert_value!("0b1010", Integer { sign: None });
    assert_value!("0o777", Integer { sign: None });
    assert_value!("0xFF", Integer { sign: None });
    assert_value!("0xDEAD_BEEF", Integer { sign: None });
    assert_value!("+1", Integer { sign: Some(Plus) });
    assert_value!("-1", Integer { sign: Some(Minus) });
    // Lexically lenient: digit validity is an eval concern.
    assert_value!("0b9", Integer { sign: None });
    assert_value!("0x", Integer { sign: None });
    // Fallbacks.
    assert_value!("42 items", LineString);
    assert_value!("-10 trailing", LineString);
    assert_value!("+ 1", LineString);
    assert_value!("+", LineString);
    assert_value!("0xG", LineString);
    assert_value!("123abc", LineString);
    assert_value!("1.5", LineString);
}

#[test]
fn classify_value_line_string_fallback() {
    assert_value!("hello world", LineString);
    assert_value!("[1, 2, 3]", LineString);
    assert_value!("/usr/local/bin", LineString);
}

#[test]
fn advance_moves_the_cursor_through_the_line() {
    let mut scanner = scan_lines("key value\n").next().unwrap().scan();
    assert_eq!(scanner.scan_word(), 3);
    scanner.advance(3);
    assert_eq!(scanner.scan_separator().space_len, 1);
    scanner.advance(1);
    assert_eq!(scanner.scan_word(), 5);
    scanner.advance(5);
    assert_eq!(scanner.rest_len(), 0);
}

#[test]
fn take_terminator_consumes_the_scanner() {
    let mut scanner = scan_lines("ab\r\n").next().unwrap().scan();
    scanner.advance(2);
    assert_eq!(scanner.take_terminator(), 2);
}

#[test]
#[should_panic(expected = "advance past the end of the line")]
fn advance_cannot_step_over_the_line_end() {
    let mut scanner = scan_lines("ab\ncd\n").next().unwrap().scan();
    scanner.advance(3);
}

#[test]
#[should_panic(expected = "line text is not fully consumed")]
fn take_terminator_requires_a_consumed_line() {
    let mut scanner = scan_lines("ab\n").next().unwrap().scan();
    scanner.advance(1);
    scanner.take_terminator();
}
