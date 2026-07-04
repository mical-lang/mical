use mical_cli_lexer::{
    LineHead::{self, *},
    Scanner,
};
use pretty_assertions::assert_eq;

macro_rules! assert_lines {
    ($src:expr, [$( ($text:literal, term: $term:literal, indent: $indent:literal, $head:ident) ),* $(,)?]) => {{
        let mut scanner = Scanner::new($src);
        $(
            let line = scanner.next_line().expect("expected more lines");
            let actual = (line.text(), line.terminator(), line.indent(), line.head());
            assert_eq!(actual, ($text, $term, $indent, $head));
        )*
        let extra_line = scanner.next_line();
        assert_eq!(extra_line, None);
    }};
}

#[test]
fn empty_source() {
    assert_lines!("", []);
}

#[test]
fn only_one_newline() {
    assert_lines!("\n", [("", term: "\n", indent: "", Blank)]);
    assert_lines!("\r", [("", term: "\r", indent: "", Blank)]);
    assert_lines!("\r\n", [("", term: "\r\n", indent: "", Blank)]);
}

#[test]
fn only_many_newlines() {
    assert_lines!("\n\n", [
        ("", term: "\n", indent: "", Blank),
        ("", term: "\n", indent: "", Blank),
    ]);
    assert_lines!("\r\r", [
        ("", term: "\r", indent: "", Blank),
        ("", term: "\r", indent: "", Blank),
    ]);
    assert_lines!("\r\n\r\n", [
        ("", term: "\r\n", indent: "", Blank),
        ("", term: "\r\n", indent: "", Blank),
    ]);
    assert_lines!("\n\r\n\r", [
        ("", term: "\n", indent: "", Blank),
        ("", term: "\r\n", indent: "", Blank),
        ("", term: "\r", indent: "", Blank),
    ]);
}

#[test]
fn lf_terminated_lines() {
    assert_lines!("a 1\nb 2\n", [
        ("a 1", term: "\n", indent: "", Text),
        ("b 2", term: "\n", indent: "", Text),
    ]);
}

#[test]
fn last_line_without_trailing_newline() {
    assert_lines!("a 1\nb 2", [
        ("a 1", term: "\n", indent: "", Text),
        ("b 2", term: "", indent: "", Text),
    ]);
}

#[test]
fn crlf_terminator_is_two_bytes() {
    assert_lines!("a 1\r\nb 2\r\n", [
        ("a 1", term: "\r\n", indent: "", Text),
        ("b 2", term: "\r\n", indent: "", Text),
    ]);
}

#[test]
fn lone_cr_is_a_terminator() {
    assert_lines!("a 1\rb 2", [
        ("a 1", term: "\r", indent: "", Text),
        ("b 2", term: "", indent: "", Text),
    ]);
    assert_lines!("a\r", [("a", term: "\r", indent: "", Text)]);
    assert_lines!("a\r\rb", [
        ("a", term: "\r", indent: "", Text),
        ("", term: "\r", indent: "", Blank),
        ("b", term: "", indent: "", Text),
    ]);
}

#[test]
fn mixed_terminators_in_one_source() {
    assert_lines!("a\nb\r\nc\rd", [
        ("a", term: "\n", indent: "", Text),
        ("b", term: "\r\n", indent: "", Text),
        ("c", term: "\r", indent: "", Text),
        ("d", term: "", indent: "", Text),
    ]);
}

#[test]
fn empty_and_space_only_lines_are_blank() {
    assert_lines!("\n   \na 1\n  ", [
        ("", term: "\n", indent: "", Blank),
        ("   ", term: "\n", indent: "   ", Blank),
        ("a 1", term: "\n", indent: "", Text),
        ("  ", term: "", indent: "  ", Blank),
    ]);
}

#[test]
fn indent_is_the_run_of_leading_spaces_only() {
    assert_lines!("  key value\n    deep\n\tx\n  \ty\n", [
        ("  key value", term: "\n", indent: "  ", Text),
        ("    deep", term: "\n", indent: "    ", Text),
        ("\tx", term: "\n", indent: "", Tab),
        ("  \ty", term: "\n", indent: "  ", Tab),
    ]);
}

#[track_caller]
fn assert_head(src: &str, expected: LineHead) {
    let head = Scanner::new(src).next_line().expect("expected a line").head();
    assert_eq!(head, expected);
}

#[test]
fn blank_head_is_empty_or_spaces_only() {
    assert_head("\n", Blank);
    assert_head("   \n", Blank);
    assert_head("   ", Blank);
}

#[test]
fn tab_head_is_a_tab_right_after_the_indent() {
    assert_head("\tx", Tab);
    assert_head("  \tx", Tab);
    assert_head("\t", Tab);
    assert_head("  \t", Tab);
}

#[test]
fn hash_head_is_a_hash_right_after_the_indent() {
    assert_head("# comment", Hash);
    assert_head("  #indented", Hash);
    assert_head("#dir x", Hash);
    assert_head("#", Hash);
}

#[test]
fn text_head_is_anything_else() {
    assert_head("a 1", Text);
    assert_head("  a", Text);
    assert_head("\"k\" v", Text);
    assert_head("|", Text);
    assert_head("42", Text);
    assert_head("こんにちは", Text);
    assert_head("a #x", Text);
    assert_head("a\tx", Text);
}

#[test]
fn a_line_partitions_into_indent_content_and_terminator() {
    let line = Scanner::new("  a b\r\n").next_line().unwrap();
    assert_eq!(line.indent(), "  ");
    assert_eq!(line.content(), "a b");
    assert_eq!(line.terminator(), "\r\n");
    assert_eq!(line.text(), "  a b");
}

#[test]
fn trailing_spaces_stay_in_the_content() {
    // Splitting off trailing spaces is layer 2's job (`Trailing::split`).
    let line = Scanner::new("a  \n").next_line().unwrap();
    assert_eq!(line.content(), "a  ");
    assert_eq!(line.terminator(), "\n");
}

#[track_caller]
fn assert_directive(src: &str, expected: bool) {
    let actual = Scanner::new(src).next_line().is_some_and(|line| line.is_directive());
    assert_eq!(actual, expected);
}

#[test]
fn directive_is_a_column_zero_hash_glued_to_a_word() {
    assert_directive("#include path", true);
    assert_directive("#!shebang", true);
    assert_directive("##", true);
    assert_directive("#1", true);
}

#[test]
fn directive_is_not_a_comment_or_an_indented_hash() {
    assert_directive("# comment", false);
    assert_directive("#", false);
    assert_directive("#\tx", false);
    assert_directive("word", false);
    assert_directive(" #x", false);
    assert_directive("  #indented x", false);
}
