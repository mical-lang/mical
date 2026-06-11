mod macros;

use mical_cli_lexer::scan_lines;

#[test]
fn empty_source_has_no_lines() {
    assert_lines!("", []);
}

#[test]
fn lf_terminated_lines() {
    assert_lines!("a 1\nb 2\n", [
        ("a 1", term: 1, indent: 0, Other),
        ("b 2", term: 1, indent: 0, Other),
    ]);
}

#[test]
fn last_line_without_trailing_newline() {
    assert_lines!("a 1\nb 2", [
        ("a 1", term: 1, indent: 0, Other),
        ("b 2", term: 0, indent: 0, Other),
    ]);
}

#[test]
fn crlf_terminator_is_two_bytes() {
    assert_lines!("a 1\r\nb 2\r\n", [
        ("a 1", term: 2, indent: 0, Other),
        ("b 2", term: 2, indent: 0, Other),
    ]);
}

#[test]
fn lone_cr_is_a_terminator() {
    assert_lines!("a 1\rb 2", [
        ("a 1", term: 1, indent: 0, Other),
        ("b 2", term: 0, indent: 0, Other),
    ]);
}

#[test]
fn empty_and_space_only_lines_are_blank() {
    assert_lines!("\n   \na 1\n", [
        ("", term: 1, indent: 0, Blank),
        ("   ", term: 1, indent: 3, Blank),
        ("a 1", term: 1, indent: 0, Other),
    ]);
}

#[test]
fn indent_counts_leading_spaces_only() {
    assert_lines!("  key value\n", [
        ("  key value", term: 1, indent: 2, Other),
    ]);
}

#[test]
fn tab_after_spaces_classifies_as_tab_head() {
    assert_lines!("\tx\n  \ty\n", [
        ("\tx", term: 1, indent: 0, Tab),
        ("  \ty", term: 1, indent: 2, Tab),
    ]);
}

#[test]
fn hash_head_classification() {
    assert_lines!("# comment\n  #indented\n#dir x\n", [
        ("# comment", term: 1, indent: 0, Hash),
        ("  #indented", term: 1, indent: 2, Hash),
        ("#dir x", term: 1, indent: 0, Hash),
    ]);
}

#[test]
fn text_excludes_the_terminator() {
    let line = scan_lines("  a b\r\n").next().unwrap();
    assert_eq!(line.text(), "  a b");
    assert_eq!(line.text_len(), 5);
    assert_eq!(line.terminator_len(), 2);
}

#[test]
fn is_directive_requires_column_zero() {
    assert_directive!("#include path");
    assert_directive!("#!shebang");
    assert_directive!(not "# comment");
    assert_directive!(not "#");
    assert_directive!(not "#\tx");
    assert_directive!(not "word");
    assert_directive!(not "  #indented x");
}
