use mical_cli_lexer::{
    Chomp::*,
    Radix::*,
    Sign::*,
    Style::*,
    ValueKind::{self, *},
};
use pretty_assertions::assert_eq;

#[track_caller]
fn assert_classify(content: &str, expected: ValueKind) {
    assert_eq!(ValueKind::classify(content), expected);
}

#[test]
fn boolean_true_needs_the_exact_word() {
    assert_classify("t", LineString);
    assert_classify("tr", LineString);
    assert_classify("tru", LineString);
    assert_classify("true", Boolean { value: true });
    assert_classify("truex", LineString);
    assert_classify("True", LineString);
    assert_classify("TRUE", LineString);
}

#[test]
fn boolean_false_needs_the_exact_word() {
    assert_classify("f", LineString);
    assert_classify("fa", LineString);
    assert_classify("fal", LineString);
    assert_classify("fals", LineString);
    assert_classify("false", Boolean { value: false });
    assert_classify("falsex", LineString);
    assert_classify("False", LineString);
}

#[test]
fn boolean_followed_by_more_content_is_a_line_string() {
    assert_classify("true value", LineString);
    assert_classify("false value", LineString);
}

#[test]
fn block_header_style_is_literal_or_folded() {
    assert_classify("|", BlockHeader { style: Literal, indent: None, chomp: None });
    assert_classify(">", BlockHeader { style: Folded, indent: None, chomp: None });
}

#[test]
fn block_header_indent_indicator_is_a_single_nonzero_digit() {
    assert_classify("|1", BlockHeader { style: Literal, indent: Some(1), chomp: None });
    assert_classify("|2", BlockHeader { style: Literal, indent: Some(2), chomp: None });
    assert_classify("|9", BlockHeader { style: Literal, indent: Some(9), chomp: None });
    assert_classify(">1", BlockHeader { style: Folded, indent: Some(1), chomp: None });
    assert_classify(">9", BlockHeader { style: Folded, indent: Some(9), chomp: None });
    assert_classify("|0", LineString);
    assert_classify(">0", LineString);
    assert_classify("|10", LineString);
}

#[test]
fn block_header_chomp_indicator_keeps_or_strips() {
    assert_classify("|+", BlockHeader { style: Literal, indent: None, chomp: Some(Keep) });
    assert_classify("|-", BlockHeader { style: Literal, indent: None, chomp: Some(Strip) });
    assert_classify(">+", BlockHeader { style: Folded, indent: None, chomp: Some(Keep) });
    assert_classify(">-", BlockHeader { style: Folded, indent: None, chomp: Some(Strip) });
}

#[test]
fn block_header_indent_precedes_chomp() {
    assert_classify("|2-", BlockHeader { style: Literal, indent: Some(2), chomp: Some(Strip) });
    assert_classify("|9+", BlockHeader { style: Literal, indent: Some(9), chomp: Some(Keep) });
    assert_classify(">1+", BlockHeader { style: Folded, indent: Some(1), chomp: Some(Keep) });
    assert_classify("|-2", LineString);
    assert_classify("|+1", LineString);
    assert_classify(">-9", LineString);
}

#[test]
fn block_header_with_trailing_junk_is_a_line_string() {
    assert_classify("|x", LineString);
    assert_classify("|2x", LineString);
    assert_classify("|+x", LineString);
    assert_classify("|2-x", LineString);
    assert_classify("||", LineString);
    assert_classify("|22", LineString);
    assert_classify("|+-", LineString);
    assert_classify("| not a block", LineString);
    assert_classify("> not folded", LineString);
}

#[test]
fn integer_decimal_digits() {
    assert_classify("0", Integer { sign: None, radix: Decimal });
    assert_classify("1", Integer { sign: None, radix: Decimal });
    assert_classify("9", Integer { sign: None, radix: Decimal });
    assert_classify("42", Integer { sign: None, radix: Decimal });
    assert_classify("00", Integer { sign: None, radix: Decimal });
    assert_classify("0123456789", Integer { sign: None, radix: Decimal });
    assert_classify("1234567890", Integer { sign: None, radix: Decimal });
}

#[test]
fn integer_underscore_needs_a_leading_digit() {
    assert_classify("1_000", Integer { sign: None, radix: Decimal });
    assert_classify("1_", Integer { sign: None, radix: Decimal });
    assert_classify("1__0", Integer { sign: None, radix: Decimal });
    assert_classify("0_", Integer { sign: None, radix: Decimal });
    assert_classify("_", LineString);
    assert_classify("_1", LineString);
}

#[test]
fn integer_binary_digits_are_zero_and_one() {
    assert_classify("0b0", Integer { sign: None, radix: Binary });
    assert_classify("0b1", Integer { sign: None, radix: Binary });
    assert_classify("0b1010", Integer { sign: None, radix: Binary });
    assert_classify("0b10_10", Integer { sign: None, radix: Binary });
    assert_classify("0b_1", Integer { sign: None, radix: Binary });
    assert_classify("0b1_", Integer { sign: None, radix: Binary });
    assert_classify("0b2", LineString);
    assert_classify("0b12", LineString);
    assert_classify("0b9", LineString);
    assert_classify("0ba", LineString);
    assert_classify("0b1a", LineString);
}

#[test]
fn integer_binary_prefix_needs_at_least_one_digit() {
    assert_classify("0b", LineString);
    assert_classify("0b_", LineString);
    assert_classify("0b__", LineString);
}

#[test]
fn integer_octal_digits_are_zero_through_seven() {
    assert_classify("0o0", Integer { sign: None, radix: Octal });
    assert_classify("0o7", Integer { sign: None, radix: Octal });
    assert_classify("0o1234567", Integer { sign: None, radix: Octal });
    assert_classify("0o12_34", Integer { sign: None, radix: Octal });
    assert_classify("0o8", LineString);
    assert_classify("0o9", LineString);
    assert_classify("0o18", LineString);
    assert_classify("0oa", LineString);
}

#[test]
fn integer_octal_prefix_needs_at_least_one_digit() {
    assert_classify("0o", LineString);
    assert_classify("0o_", LineString);
}

#[test]
fn integer_hexadecimal_digits_include_both_letter_cases() {
    assert_classify("0x0", Integer { sign: None, radix: Hexadecimal });
    assert_classify("0x9", Integer { sign: None, radix: Hexadecimal });
    assert_classify("0xa", Integer { sign: None, radix: Hexadecimal });
    assert_classify("0xf", Integer { sign: None, radix: Hexadecimal });
    assert_classify("0xA", Integer { sign: None, radix: Hexadecimal });
    assert_classify("0xF", Integer { sign: None, radix: Hexadecimal });
    assert_classify("0x1234567890abcdef", Integer { sign: None, radix: Hexadecimal });
    assert_classify("0x1234567890ABCDEF", Integer { sign: None, radix: Hexadecimal });
    assert_classify("0xDEAD_BEEF", Integer { sign: None, radix: Hexadecimal });
    assert_classify("0xg", LineString);
    assert_classify("0xG", LineString);
    assert_classify("0x1g", LineString);
}

#[test]
fn integer_hexadecimal_prefix_needs_at_least_one_digit() {
    assert_classify("0x", LineString);
    assert_classify("0x_", LineString);
}

#[test]
fn integer_radix_prefix_is_lowercase_after_a_single_zero() {
    assert_classify("0B1", LineString);
    assert_classify("0O7", LineString);
    assert_classify("0X1", LineString);
    assert_classify("00b1", LineString);
    assert_classify("1b1", LineString);
    assert_classify("b1", LineString);
}

#[test]
fn integer_sign_must_be_adjacent_to_the_numeral() {
    assert_classify("+1", Integer { sign: Some(Plus), radix: Decimal });
    assert_classify("-1", Integer { sign: Some(Minus), radix: Decimal });
    assert_classify("+0", Integer { sign: Some(Plus), radix: Decimal });
    assert_classify("-0", Integer { sign: Some(Minus), radix: Decimal });
    assert_classify("+1_000", Integer { sign: Some(Plus), radix: Decimal });
    assert_classify("+ 1", LineString);
    assert_classify("+", LineString);
    assert_classify("-", LineString);
}

#[test]
fn integer_sign_applies_to_any_radix() {
    assert_classify("+0b1", Integer { sign: Some(Plus), radix: Binary });
    assert_classify("-0b1", Integer { sign: Some(Minus), radix: Binary });
    assert_classify("+0o7", Integer { sign: Some(Plus), radix: Octal });
    assert_classify("-0o7", Integer { sign: Some(Minus), radix: Octal });
    assert_classify("+0xF", Integer { sign: Some(Plus), radix: Hexadecimal });
    assert_classify("-0xF", Integer { sign: Some(Minus), radix: Hexadecimal });
}

#[test]
fn integer_takes_at_most_one_sign() {
    assert_classify("++1", LineString);
    assert_classify("--1", LineString);
    assert_classify("+-1", LineString);
    assert_classify("-+1", LineString);
}

#[test]
fn integer_sign_before_a_non_numeral_is_a_line_string() {
    assert_classify("+true", LineString);
    assert_classify("-false", LineString);
    assert_classify("+_1", LineString);
    assert_classify("-a", LineString);
}

#[test]
fn integer_followed_by_more_content_is_a_line_string() {
    assert_classify("42 items", LineString);
    assert_classify("-10 trailing", LineString);
    assert_classify("123abc", LineString);
}

#[test]
fn non_integer_number_formats_are_line_strings() {
    assert_classify("1.5", LineString);
    assert_classify("0.0", LineString);
    assert_classify("1e3", LineString);
    assert_classify("1,000", LineString);
}

#[test]
fn line_string_is_the_fallback_for_everything_else() {
    assert_classify("", LineString);
    assert_classify("hello", LineString);
    assert_classify("hello world", LineString);
    assert_classify("[1, 2, 3]", LineString);
    assert_classify("/usr/local/bin", LineString);
    assert_classify("null", LineString);
    assert_classify("value \"quoted\" text", LineString);
    assert_classify("こんにちは", LineString);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "value content must be pre-split from its trailing part")]
fn classify_rejects_content_with_a_trailing_part() {
    let _ = ValueKind::classify("value ");
}
