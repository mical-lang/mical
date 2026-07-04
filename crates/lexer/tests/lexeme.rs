use mical_cli_lexer::{
    Lexeme as _,
    Quote::{self, *},
    Quoted, Scanner, Separator, Trailing, Word,
};
use pretty_assertions::assert_eq;

#[track_caller]
fn assert_word(src: &str, expected: Option<&str>) {
    assert_eq!(Word::at(src).map(|w| w.text()), expected);
}

#[test]
fn word_runs_until_space_tab_or_end() {
    assert_word("hello world", Some("hello"));
    assert_word("key\tvalue", Some("key"));
    assert_word("a  b", Some("a"));
    assert_word("alone", Some("alone"));
}

#[test]
fn word_takes_punctuation_digits_and_multibyte() {
    assert_word("server.port 8080", Some("server.port"));
    assert_word("a#b value", Some("a#b"));
    assert_word("can't value", Some("can't"));
    assert_word("{a}|b>c value", Some("{a}|b>c"));
    assert_word("-x+y value", Some("-x+y"));
    assert_word("42 value", Some("42"));
    assert_word("true value", Some("true"));
    // Quote priority is the caller's concern: Word itself is quote-agnostic.
    assert_word("\"a b\" c", Some("\"a"));
    assert_word("こんにちは こんばんは", Some("こんにちは"));
    assert_word("🐰👑", Some("🐰👑"));
}

#[test]
fn word_is_absent_at_whitespace_or_end() {
    assert_word("", None);
    assert_word(" leading", None);
    assert_word("\tx", None);
}

#[track_caller]
fn assert_quoted(src: &str, expected: Option<(Quote, &str, bool)>) {
    let actual = Quoted::at(src);
    assert_eq!(actual.map(|q| (q.quote(), q.content(), q.is_closed())), expected);
    if let Some(quoted) = actual {
        assert!(src.starts_with(quoted.text()), "text must be a prefix of the input");
    }
}

#[test]
fn quoted_grows_from_the_open_quote_to_the_matching_close() {
    assert_quoted("\"", Some((Double, "", false)));
    assert_quoted("\"\"", Some((Double, "", true)));
    assert_quoted("\"a", Some((Double, "a", false)));
    assert_quoted("\"a\"", Some((Double, "a", true)));
    assert_quoted("'", Some((Single, "", false)));
    assert_quoted("''", Some((Single, "", true)));
    assert_quoted("'a", Some((Single, "a", false)));
    assert_quoted("'a'", Some((Single, "a", true)));
}

#[test]
fn quoted_stops_at_the_first_matching_close() {
    assert_quoted("\"double\" v", Some((Double, "double", true)));
    assert_quoted("'single' v", Some((Single, "single", true)));
    assert_quoted("\"with space\" v", Some((Double, "with space", true)));
    assert_quoted("\"a\"b\"", Some((Double, "a", true)));
    assert_quoted("\"quoted\"ppp value", Some((Double, "quoted", true)));
}

#[test]
fn quoted_treats_the_other_quote_as_content() {
    assert_quoted("\"it's\" v", Some((Double, "it's", true)));
    assert_quoted("'say \"hi\"' v", Some((Single, "say \"hi\"", true)));
    assert_quoted("\"''\" v", Some((Double, "''", true)));
}

#[test]
fn quoted_hash_and_multibyte_are_literal_content() {
    assert_quoted("\"a # b\" v", Some((Double, "a # b", true)));
    assert_quoted("'こんにちは' v", Some((Single, "こんにちは", true)));
}

#[test]
fn quoted_unclosed_runs_to_the_end() {
    assert_quoted("\"unclosed value", Some((Double, "unclosed value", false)));
    assert_quoted("'unclosed value", Some((Single, "unclosed value", false)));
}

#[test]
fn quoted_is_absent_unless_it_starts_with_a_quote() {
    assert_quoted("plain", None);
    assert_quoted("", None);
    assert_quoted(" \"padded\"", None);
    assert_quoted("\\\"escaped\"", None);
}

#[test]
fn quoted_escaped_matching_quote_does_not_close() {
    assert_quoted(r#""a\"b" v"#, Some((Double, r#"a\"b"#, true)));
    assert_quoted(r"'a\'b' v", Some((Single, r"a\'b", true)));
    assert_quoted(r#""\""#, Some((Double, r#"\""#, false)));
    assert_quoted(r"'\'", Some((Single, r"\'", false)));
}

#[test]
fn quoted_escaped_backslash_does_not_protect_the_close() {
    assert_quoted(r#""a\\" v"#, Some((Double, r"a\\", true)));
    assert_quoted(r"'\\' v", Some((Single, r"\\", true)));
}

#[test]
fn quoted_backslash_before_other_characters_is_plain_content() {
    // Escape validity (e.g. `\n` vs `\z`) is judged later by the evaluator;
    // scanning only cares about `\\` and the matching quote.
    assert_quoted(r#""a\nb" v"#, Some((Double, r"a\nb", true)));
    assert_quoted(r#""a\zb" v"#, Some((Double, r"a\zb", true)));
    // `\"` inside single quotes is not an escape that matters for closing.
    assert_quoted(r#"'a\"b' v"#, Some((Single, r#"a\"b"#, true)));
    // A trailing lone backslash cannot escape past the end.
    assert_quoted(r#""a\"#, Some((Double, r"a\", false)));
}

#[test]
fn quoted_text_covers_the_quotes() {
    let closed = Quoted::at("\"abc\" tail").unwrap();
    assert_eq!(closed.text(), "\"abc\"");
    let unclosed = Quoted::at("\"abc tail").unwrap();
    assert_eq!(unclosed.text(), "\"abc tail");
}

#[track_caller]
fn assert_separator(src: &str, expected: Option<(&str, Option<&str>)>) {
    let actual = Separator::at(src);
    assert_eq!(actual.map(|s| (s.spaces(), s.tab_run())), expected);
    if let Some(separator) = actual {
        let parts = format!("{}{}", separator.spaces(), separator.tab_run().unwrap_or(""));
        assert_eq!(separator.text(), parts);
        assert!(src.starts_with(separator.text()), "text must be a prefix of the input");
    }
}

#[test]
fn separator_takes_the_run_of_spaces() {
    assert_separator(" v", Some((" ", None)));
    assert_separator("   v", Some(("   ", None)));
    assert_separator(" ", Some((" ", None)));
}

#[test]
fn separator_tab_run_is_tabs_and_spaces_after_the_first_tab() {
    assert_separator("\tv", Some(("", Some("\t"))));
    assert_separator("\t\tv", Some(("", Some("\t\t"))));
    assert_separator(" \tv", Some((" ", Some("\t"))));
    assert_separator("  \t v", Some(("  ", Some("\t "))));
    assert_separator("\t \t v", Some(("", Some("\t \t "))));
    assert_separator(" \t", Some((" ", Some("\t"))));
}

#[test]
fn separator_is_absent_without_leading_whitespace() {
    assert_separator("v", None);
    assert_separator("v ", None);
    assert_separator("", None);
}

#[track_caller]
fn assert_trailing(src: &str, content: &str, trailing: Option<Trailing>) {
    let actual = Trailing::split(src);
    assert_eq!(actual, (content, trailing));
    let reconstructed = format!("{}{}", actual.0, actual.1.map_or("", |t| t.text()));
    assert_eq!(reconstructed, src, "split must account for every byte");
}

#[test]
fn trailing_comment_starts_at_a_hash_at_position_zero_or_after_a_space() {
    assert_trailing("value # c", "value", Some(Trailing::Comment(" # c")));
    assert_trailing("value #c", "value", Some(Trailing::Comment(" #c")));
    assert_trailing("value #", "value", Some(Trailing::Comment(" #")));
    assert_trailing("# full line", "", Some(Trailing::Comment("# full line")));
    assert_trailing("#bare", "", Some(Trailing::Comment("#bare")));
}

#[test]
fn trailing_comment_absorbs_the_spaces_before_the_hash() {
    assert_trailing("value   # c", "value", Some(Trailing::Comment("   # c")));
    assert_trailing("   # c", "", Some(Trailing::Comment("   # c")));
}

#[test]
fn trailing_hash_not_preceded_by_a_space_is_content() {
    assert_trailing("hello#world", "hello#world", None);
    assert_trailing("a#b #c", "a#b", Some(Trailing::Comment(" #c")));
    // A tab does not start a comment.
    assert_trailing("x\t#y", "x\t#y", None);
}

#[test]
fn trailing_splits_at_the_first_positional_hash() {
    assert_trailing("a # b # c", "a", Some(Trailing::Comment(" # b # c")));
    // Mid-line quotes do not protect '#'.
    assert_trailing("a \"x # y\" tail", "a \"x", Some(Trailing::Comment(" # y\" tail")));
}

#[test]
fn trailing_spaces_split_off_without_a_comment() {
    assert_trailing("value", "value", None);
    assert_trailing("value ", "value", Some(Trailing::Spaces(" ")));
    assert_trailing("value   ", "value", Some(Trailing::Spaces("   ")));
    assert_trailing("   ", "", Some(Trailing::Spaces("   ")));
    // Only spaces are trailing; a tab stays in the content.
    assert_trailing("value\t", "value\t", None);
    assert_trailing("", "", None);
}

#[test]
fn take_walks_the_line_through_lexemes() {
    let line = Scanner::new("key value\n").next_line().unwrap();
    let mut rest = line.content();
    assert_eq!(Word::take(&mut rest).unwrap().text(), "key");
    assert_eq!(Separator::take(&mut rest).unwrap().spaces(), " ");
    assert_eq!(Word::take(&mut rest).unwrap().text(), "value");
    assert_eq!(rest, "");
}

#[test]
fn take_consumes_exactly_the_lexeme_text() {
    let mut rest = "'a b' tail";
    assert_eq!(Quoted::take(&mut rest).unwrap().text(), "'a b'");
    assert_eq!(rest, " tail");
}

#[test]
fn take_leaves_rest_untouched_when_the_lexeme_is_absent() {
    let mut rest = "key value";
    assert_eq!(Separator::take(&mut rest), None);
    assert_eq!(rest, "key value");
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "input must be text within a single line")]
fn at_rejects_text_spanning_lines() {
    let _ = Word::at("a\nb");
}
