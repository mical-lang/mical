mod macros;
use mical_syntax::token::{Quote::*, Radix::*};

#[test]
fn true_() {
    assert_token!("t", [Word(1)]);
    assert_token!("tr", [Word(2)]);
    assert_token!("tru", [Word(3)]);
    assert_token!("true", [True(4)]);
    assert_token!("truex", [True(4), Word(1)]);
}

#[test]
fn false_() {
    assert_token!("f", [Word(1)]);
    assert_token!("fa", [Word(2)]);
    assert_token!("fal", [Word(3)]);
    assert_token!("fals", [Word(4)]);
    assert_token!("false", [False(5)]);
    assert_token!("falsex", [False(5), Word(1)]);
}

#[test]
fn integer_binary() {
    assert_token!("0b", [Numeral { 2, radix: Binary, is_empty: true }]);
    assert_token!("0b_", [Numeral { 3, radix: Binary, is_empty: true }]);
    assert_token!("0b0", [Numeral { 3, radix: Binary, is_empty: false }]);
    assert_token!("0b0_", [Numeral { 4, radix: Binary, is_empty: false }]);
    assert_token!("0b1010", [Numeral { 6, radix: Binary, is_empty: false }]);
    assert_token!("0b0101", [Numeral { 6, radix: Binary, is_empty: false }]);
    assert_token!("0b123456789", [Numeral { 11, radix: Binary, is_empty: false }]);
    assert_token!("0b10_10", [Numeral { 7, radix: Binary, is_empty: false }]);
    assert_token!("0ba", [Word(3)]);
    assert_token!("0b1a", [Word(4)]);
    assert_token!("a0b1", [Word(4)]);
}

#[test]
fn integer_octal() {
    assert_token!("0o", [Numeral { 2, radix: Octal, is_empty: true }]);
    assert_token!("0o_", [Numeral { 3, radix: Octal, is_empty: true }]);
    assert_token!("0o0", [Numeral { 3, radix: Octal, is_empty: false }]);
    assert_token!("0o0_", [Numeral { 4, radix: Octal, is_empty: false }]);
    assert_token!("0o1234567", [Numeral { 9, radix: Octal, is_empty: false }]);
    assert_token!("0o123456789", [Numeral { 11, radix: Octal, is_empty: false }]);
    assert_token!("0o12_34_56", [Numeral { 10, radix: Octal, is_empty: false }]);
    assert_token!("0oa", [Word(3)]);
    assert_token!("0o8a", [Word(4)]);
    assert_token!("a0o1", [Word(4)]);
}

#[test]
fn integer_hexadecimal() {
    assert_token!("0x", [Numeral { 2, radix: Hexadecimal, is_empty: true }]);
    assert_token!("0x_", [Numeral { 3, radix: Hexadecimal, is_empty: true }]);
    assert_token!("0x0", [Numeral { 3, radix: Hexadecimal, is_empty: false }]);
    assert_token!("0x0_", [Numeral { 4, radix: Hexadecimal, is_empty: false }]);
    assert_token!("0x1234567890ABCDEF", [Numeral { 18, radix: Hexadecimal, is_empty: false }]);
    assert_token!("0x1234567890abcdef", [Numeral { 18, radix: Hexadecimal, is_empty: false }]);
    assert_token!("0x12_34_56_ab_cd_EF", [Numeral { 19, radix: Hexadecimal, is_empty: false }]);
    assert_token!("0xg", [Word(3)]);
    assert_token!("0xfg", [Word(4)]);
    assert_token!("a0x1", [Word(4)]);
}

#[test]
fn integer_decimal() {
    assert_token!("0", [Numeral { 1, radix: Decimal, is_empty: false }]);
    assert_token!("00", [Numeral { 2, radix: Decimal, is_empty: false }]);
    assert_token!("_", [Word(1)]);
    assert_token!("_0", [Word(2)]);
    assert_token!("0123456789", [Numeral { 10, radix: Decimal, is_empty: false }]);
    assert_token!("1234567890", [Numeral { 10, radix: Decimal, is_empty: false }]);
    assert_token!("0123456789_", [Numeral { 11, radix: Decimal, is_empty: false }]);
    assert_token!("01234_56789", [Numeral { 11, radix: Decimal, is_empty: false }]);
    assert_token!("0a", [Word(2)]);
    assert_token!("123a", [Word(4)]);
    assert_token!("a1", [Word(2)]);
}

#[test]
fn single_punctuation() {
    assert_token!("{", [OpenBrace(1)]);
    assert_token!("}", [CloseBrace(1)]);
    assert_token!(">", [Greater(1)]);
    assert_token!("-", [Minus(1)]);
    assert_token!("|", [Pipe(1)]);
    assert_token!("+", [Plus(1)]);
    assert_token!("#", [Sharp(1)]);
}

#[test]
fn multiple_punctuation() {
    assert_token!("{{", [OpenBrace(1), OpenBrace(1)]);
    assert_token!("{ {", [OpenBrace(1), Space(1), OpenBrace(1)]);
    assert_token!("}}", [CloseBrace(1), CloseBrace(1)]);
    assert_token!("} }", [CloseBrace(1), Space(1), CloseBrace(1)]);
    assert_token!(">>", [Greater(1), Greater(1)]);
    assert_token!("> >", [Greater(1), Space(1), Greater(1)]);
    assert_token!("--", [Minus(1), Minus(1)]);
    assert_token!("- -", [Minus(1), Space(1), Minus(1)]);
    assert_token!("||", [Pipe(1), Pipe(1)]);
    assert_token!("| |", [Pipe(1), Space(1), Pipe(1)]);
    assert_token!("++", [Plus(1), Plus(1)]);
    assert_token!("+ +", [Plus(1), Space(1), Plus(1)]);
    assert_token!("##", [Sharp(1), Sharp(1)]);
    assert_token!("# #", [Sharp(1), Space(1), Sharp(1)]);
}

#[test]
fn single_whitespace() {
    assert_token!("\t", [Tab(1)]);
    assert_token!("\n", [Newline(1)]);
    assert_token!(" ", [Space(1)]);
    assert_token!("\r", [Newline(1)]);
    assert_token!("\r\n", [Newline(2)]);
}

#[test]
fn multiple_whitespace() {
    assert_token!("\t\t", [Tab(2)]);
    assert_token!("\n\n", [Newline(1), Newline(1)]);
    assert_token!("  ", [Space(2)]);
    assert_token!("\r\r", [Newline(1), Newline(1)]);
    assert_token!("\r\n\r\n", [Newline(2), Newline(2)]);
}

#[test]
fn multibyte_word() {
    assert_token!("こんにちは", [Word(15)]);
    assert_token!("你好", [Word(6)]);
    assert_token!("안녕하세요", [Word(15)]);
    assert_token!("🐰👑", [Word(8)]);
}

#[test]
fn punctuation_first_word() {
    assert_token!("{x", [OpenBrace(1), Word(1)]);
    assert_token!("}x", [CloseBrace(1), Word(1)]);
    assert_token!(">x", [Greater(1), Word(1)]);
    assert_token!("-x", [Minus(1), Word(1)]);
    assert_token!("|x", [Pipe(1), Word(1)]);
    assert_token!("+x", [Plus(1), Word(1)]);
    assert_token!("#x", [Sharp(1), Word(1)]);
    assert_token!("'x", [String { 2, is_terminated: false, quote: Single }]);
    assert_token!("\"x", [String { 2, is_terminated: false, quote: Double }]);
}

#[test]
fn punctuation_last_word() {
    assert_token!("x{", [Word(2)]);
    assert_token!("x}", [Word(2)]);
    assert_token!("x>", [Word(2)]);
    assert_token!("x-", [Word(2)]);
    assert_token!("x|", [Word(2)]);
    assert_token!("x+", [Word(2)]);
    assert_token!("x#", [Word(2)]);
    assert_token!("x'", [Word(2)]);
    assert_token!("x\"", [Word(2)]);
}

#[test]
fn punctuation_middle_word() {
    assert_token!("x{y", [Word(3)]);
    assert_token!("x}y", [Word(3)]);
    assert_token!("x>y", [Word(3)]);
    assert_token!("x-y", [Word(3)]);
    assert_token!("x|y", [Word(3)]);
    assert_token!("x+y", [Word(3)]);
    assert_token!("x#y", [Word(3)]);
    assert_token!("x'y", [Word(3)]);
    assert_token!("x\"y", [Word(3)]);
}

#[test]
fn empty_string() {
    assert_token!(r#"""#, [String { 1, is_terminated: false, quote: Double }]);
    assert_token!(r#"'"#, [String { 1, is_terminated: false, quote: Single }]);
    assert_token!(r#"''"#, [String { 2, is_terminated: true, quote: Single }]);
    assert_token!(r#""""#, [String { 2, is_terminated: true, quote: Double }]);
}

#[test]
fn string() {
    assert_token!(r#""'""#, [String { 3, is_terminated: true, quote: Double }]);
    assert_token!(r#""''""#, [String { 4, is_terminated: true, quote: Double }]);
    assert_token!(r#"'""'"#, [String { 4, is_terminated: true, quote: Single }]);
    assert_token!(r#"' '"#, [String { 3, is_terminated: true, quote: Single }]);
    assert_token!(r#"" ""#, [String { 3, is_terminated: true, quote: Double }]);
    assert_token!(r#"'foo'"#, [String { 5, is_terminated: true, quote: Single }]);
    assert_token!(r#""bar""#, [String { 5, is_terminated: true, quote: Double }]);
    assert_token!(r#"'a b'"#, [String { 5, is_terminated: true, quote: Single }]);
    assert_token!(r#""c d""#, [String { 5, is_terminated: true, quote: Double }]);
    assert_token!(
        "'pi\nyo'",
        [String { 3, is_terminated: false, quote: Single }, Newline(1), Word(3)]
    );
    assert_token!(
        "\"pi\r\nyo\"",
        [String { 3, is_terminated: false, quote: Double }, Newline(2), Word(3)]
    );
}

#[test]
fn escaped_string() {
    assert_token!(r#"'\'"#, [String { 3, is_terminated: false, quote: Single }]);
    assert_token!(r#"'\''"#, [String { 4, is_terminated: true, quote: Single }]);
    assert_token!(r#""\""#, [String { 3, is_terminated: false, quote: Double }]);
    assert_token!(r#""\"""#, [String { 4, is_terminated: true, quote: Double }]);
    assert_token!(r#""hoge\"fuga""#, [String { 12, is_terminated: true, quote: Double }]);
    assert_token!(r#"'bar\'baz'"#, [String { 10, is_terminated: true, quote: Single }]);
    assert_token!(
        "'\\\n'",
        [
            String { 2, is_terminated: false, quote: Single },
            Newline(1),
            String { 1, is_terminated: false, quote: Single },
        ]
    );
    assert_token!(
        "\"\\\r\n\"",
        [
            String { 2, is_terminated: false, quote: Double },
            Newline(2),
            String { 1, is_terminated: false, quote: Double },
        ]
    );
}
