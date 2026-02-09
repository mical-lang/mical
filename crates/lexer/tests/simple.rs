mod macros;
use mical_syntax::token::NumBase::*;

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
    assert_token!("0b", [Integer { 2, base: Binary, is_empty: true }]);
    assert_token!("0b_", [Integer { 3, base: Binary, is_empty: true }]);
    assert_token!("0b0", [Integer { 3, base: Binary, is_empty: false }]);
    assert_token!("0b1010", [Integer { 6, base: Binary, is_empty: false }]);
    assert_token!("0b0101", [Integer { 6, base: Binary, is_empty: false }]);
    assert_token!("0b123456789", [Integer { 11, base: Binary, is_empty: false }]);
    assert_token!("0b10_10", [Integer { 7, base: Binary, is_empty: false }]);
    assert_token!("0ba", [Integer { 2, base: Binary, is_empty: true }, Word(1)]);
    assert_token!("0b1a", [Integer { 3, base: Binary, is_empty: false }, Word(1)]);
}

#[test]
fn integer_octal() {
    assert_token!("0o", [Integer { 2, base: Octal, is_empty: true }]);
    assert_token!("0o_", [Integer { 3, base: Octal, is_empty: true }]);
    assert_token!("0o0", [Integer { 3, base: Octal, is_empty: false }]);
    assert_token!("0o1234567", [Integer { 9, base: Octal, is_empty: false }]);
    assert_token!("0o123456789", [Integer { 11, base: Octal, is_empty: false }]);
    assert_token!("0o12_34_56", [Integer { 10, base: Octal, is_empty: false }]);
    assert_token!("0oa", [Integer { 2, base: Octal, is_empty: true }, Word(1)]);
    assert_token!("0o8a", [Integer { 3, base: Octal, is_empty: false }, Word(1)]);
}

#[test]
fn integer_hexadecimal() {
    assert_token!("0x", [Integer { 2, base: Hexadecimal, is_empty: true }]);
    assert_token!("0x_", [Integer { 3, base: Hexadecimal, is_empty: true }]);
    assert_token!("0x0", [Integer { 3, base: Hexadecimal, is_empty: false }]);
    assert_token!("0x1234567890ABCDEF", [Integer { 18, base: Hexadecimal, is_empty: false }]);
    assert_token!("0x1234567890abcdef", [Integer { 18, base: Hexadecimal, is_empty: false }]);
    assert_token!("0x12_34_56_ab_cd_EF", [Integer { 19, base: Hexadecimal, is_empty: false }]);
    assert_token!("0xg", [Integer { 2, base: Hexadecimal, is_empty: true }, Word(1)]);
    assert_token!("0xfg", [Integer { 3, base: Hexadecimal, is_empty: false }, Word(1)]);
}

#[test]
fn integer_decimal() {
    assert_token!("0", [Integer { 1, base: Decimal, is_empty: false }]);
    assert_token!("00", [Integer { 2, base: Decimal, is_empty: false }]);
    assert_token!("_", [Word(1)]);
    assert_token!("_0", [Word(2)]);
    assert_token!("0123456789", [Integer { 10, base: Decimal, is_empty: false }]);
    assert_token!("1234567890", [Integer { 10, base: Decimal, is_empty: false }]);
    assert_token!("0123456789_", [Integer { 11, base: Decimal, is_empty: false }]);
    assert_token!("01234_56789", [Integer { 11, base: Decimal, is_empty: false }]);
    assert_token!("0a", [Integer { 1, base: Decimal, is_empty: false }, Word(1)]);
    assert_token!("123a", [Integer { 3, base: Decimal, is_empty: false }, Word(1)]);
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
            Integer { 2, base: Decimal, is_empty: false },
            Backslash(1),
            Integer { 2, base: Decimal, is_empty: false },
        ]
    );
}
