mod macros;
use mical_syntax::token::Radix::*;

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
    assert_token!("0b1010", [Numeral { 6, radix: Binary, is_empty: false }]);
    assert_token!("0b0101", [Numeral { 6, radix: Binary, is_empty: false }]);
    assert_token!("0b123456789", [Numeral { 11, radix: Binary, is_empty: false }]);
    assert_token!("0b10_10", [Numeral { 7, radix: Binary, is_empty: false }]);
    assert_token!("0ba", [Numeral { 2, radix: Binary, is_empty: true }, Word(1)]);
    assert_token!("0b1a", [Numeral { 3, radix: Binary, is_empty: false }, Word(1)]);
}

#[test]
fn integer_octal() {
    assert_token!("0o", [Numeral { 2, radix: Octal, is_empty: true }]);
    assert_token!("0o_", [Numeral { 3, radix: Octal, is_empty: true }]);
    assert_token!("0o0", [Numeral { 3, radix: Octal, is_empty: false }]);
    assert_token!("0o1234567", [Numeral { 9, radix: Octal, is_empty: false }]);
    assert_token!("0o123456789", [Numeral { 11, radix: Octal, is_empty: false }]);
    assert_token!("0o12_34_56", [Numeral { 10, radix: Octal, is_empty: false }]);
    assert_token!("0oa", [Numeral { 2, radix: Octal, is_empty: true }, Word(1)]);
    assert_token!("0o8a", [Numeral { 3, radix: Octal, is_empty: false }, Word(1)]);
}

#[test]
fn integer_hexadecimal() {
    assert_token!("0x", [Numeral { 2, radix: Hexadecimal, is_empty: true }]);
    assert_token!("0x_", [Numeral { 3, radix: Hexadecimal, is_empty: true }]);
    assert_token!("0x0", [Numeral { 3, radix: Hexadecimal, is_empty: false }]);
    assert_token!("0x1234567890ABCDEF", [Numeral { 18, radix: Hexadecimal, is_empty: false }]);
    assert_token!("0x1234567890abcdef", [Numeral { 18, radix: Hexadecimal, is_empty: false }]);
    assert_token!("0x12_34_56_ab_cd_EF", [Numeral { 19, radix: Hexadecimal, is_empty: false }]);
    assert_token!("0xg", [Numeral { 2, radix: Hexadecimal, is_empty: true }, Word(1)]);
    assert_token!("0xfg", [Numeral { 3, radix: Hexadecimal, is_empty: false }, Word(1)]);
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
    assert_token!("0a", [Numeral { 1, radix: Decimal, is_empty: false }, Word(1)]);
    assert_token!("123a", [Numeral { 3, radix: Decimal, is_empty: false }, Word(1)]);
}

#[test]
fn single_punctuation() {
    assert_token!(r"\", [Backslash(1)]);
    assert_token!("{", [OpenBrace(1)]);
    assert_token!("}", [CloseBrace(1)]);
    assert_token!(">", [Greater(1)]);
    assert_token!("-", [Minus(1)]);
    assert_token!("|", [Pipe(1)]);
    assert_token!("+", [Plus(1)]);
    assert_token!("#", [Sharp(1)]);
}

#[test]
fn simgle_whitespace() {
    assert_token!("\t", [Tab(1)]);
    assert_token!("\n", [Newline(1)]);
    assert_token!(" ", [Space(1)]);
}

#[test]
fn backslash() {
    assert_token!(r"\", [Backslash(1)]);
    assert_token!(r"\\", [Backslash(1), Backslash(1)]);
    assert_token!(r"\a", [Backslash(1), Word(1)]);
    assert_token!(r"\n", [Backslash(1), Word(1)]);
    assert_token!(r"\ ", [Backslash(1), Space(1)]);
    assert_token!(r"a\b", [Word(1), Backslash(1), Word(1)]);
    assert_token!(r"a\\b", [Word(1), Backslash(1), Backslash(1), Word(1)]);
    assert_token!(
        r"01\23",
        [
            Numeral { 2, radix: Decimal, is_empty: false },
            Backslash(1),
            Numeral { 2, radix: Decimal, is_empty: false },
        ]
    );
}
