use mical_syntax::token::{TokenKind::*, *};
use std::iter;

mod cursor;
use cursor::Cursor;

struct TokenStreamImpl<'src, I: Iterator<Item = Token>> {
    source: &'src str,
    iter: I,
}

impl<I: Iterator<Item = Token>> Iterator for TokenStreamImpl<'_, I> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
impl<'s, I: Iterator<Item = Token>> TokenStream<'s> for TokenStreamImpl<'s, I> {
    fn source(&self) -> &'s str {
        self.source
    }
}

pub fn tokenize(source: &str) -> impl TokenStream<'_> {
    let mut cursor = Cursor::new(source);
    TokenStreamImpl { source, iter: iter::from_fn(move || advance_token(&mut cursor)) }
}

fn advance_token(cursor: &mut Cursor) -> Option<Token> {
    let kind = match cursor.next()? {
        't' => true_or_word(cursor),
        'f' => false_or_word(cursor),
        '\t' => {
            cursor.eat_while(|c| c == '\t');
            Tab
        }
        '\n' => Newline,
        '\r' => {
            if let Some('\n') = cursor.peek() {
                cursor.next();
            }
            Newline
        }
        ' ' => {
            cursor.eat_while(|c| c == ' ');
            Space
        }
        '}' => CloseBrace,
        '>' => Greater,
        '-' => Minus,
        '{' => OpenBrace,
        '|' => Pipe,
        '+' => Plus,
        '#' => Sharp,
        '"' => string::<'"'>(cursor),
        '\'' => string::<'\''>(cursor),
        c @ '0'..='9' => integer_or_word(cursor, c),
        _ => word(cursor),
    };
    let token = cursor.bump(kind);
    Some(token)
}

fn true_or_word(cursor: &mut Cursor) -> TokenKind {
    debug_assert!(cursor.prev() == 't');
    if let Some('r') = cursor.peek() {
        cursor.next();
        if let Some('u') = cursor.peek() {
            cursor.next();
            if let Some('e') = cursor.peek() {
                cursor.next();
                return True;
            }
        }
    }
    word(cursor)
}

fn false_or_word(cursor: &mut Cursor) -> TokenKind {
    debug_assert!(cursor.prev() == 'f');
    if let Some('a') = cursor.peek() {
        cursor.next();
        if let Some('l') = cursor.peek() {
            cursor.next();
            if let Some('s') = cursor.peek() {
                cursor.next();
                if let Some('e') = cursor.peek() {
                    cursor.next();
                    return False;
                }
            }
        }
    }
    word(cursor)
}

fn string<const Q: char>(cursor: &mut Cursor) -> TokenKind {
    const { assert!(Q == '"' || Q == '\'') };
    debug_assert!(cursor.prev() == Q);

    let mut terminated = false;
    while let Some(c) = cursor.peek() {
        match c {
            '\\' => {
                cursor.next();
                let peek = cursor.peek();
                if peek == Some(Q) || peek == Some('\\') {
                    cursor.next();
                }
            }
            '\n' | '\r' => {
                break;
            }
            q if q == Q => {
                terminated = true;
                cursor.next();
                break;
            }
            _ => {
                cursor.next();
            }
        }
    }
    String {
        is_terminated: terminated,
        quote: const {
            match Q {
                '"' => Quote::Double,
                '\'' => Quote::Single,
                _ => unreachable!(),
            }
        },
    }
}

fn integer_or_word(cursor: &mut Cursor, first_digit: char) -> TokenKind {
    debug_assert!(first_digit.is_ascii_digit()); // 0..=9
    fn eat_decimal_digits(cursor: &mut Cursor) -> bool {
        let mut has_digits = false;
        while let Some(c) = cursor.peek() {
            match c {
                '_' => (),
                '0'..='9' => has_digits = true,
                _ => break,
            };
            cursor.next();
        }
        has_digits
    }
    fn eat_hexadecimal_digits(cursor: &mut Cursor) -> bool {
        let mut has_digits = false;
        while let Some(c) = cursor.peek() {
            match c {
                '_' => (),
                '0'..='9' | 'a'..='f' | 'A'..='F' => has_digits = true,
                _ => break,
            };
            cursor.next();
        }
        has_digits
    }
    let mut radix = Radix::Decimal;
    let has_digits = if first_digit == '0' {
        match cursor.peek() {
            Some('b') => {
                radix = Radix::Binary;
                cursor.next();
                eat_decimal_digits(cursor)
            }
            Some('o') => {
                radix = Radix::Octal;
                cursor.next();
                eat_decimal_digits(cursor)
            }
            Some('x') => {
                radix = Radix::Hexadecimal;
                cursor.next();
                eat_hexadecimal_digits(cursor)
            }
            Some('0'..='9' | '_') => eat_decimal_digits(cursor),
            _ => true, // single '0'
        }
    } else {
        eat_decimal_digits(cursor)
    };
    match cursor.peek() {
        Some('\t' | '\n' | ' ') | None => Numeral { radix, is_empty: !has_digits },
        _ => word(cursor),
    }
}

fn word(cursor: &mut Cursor) -> TokenKind {
    cursor.eat_while(|c| !matches!(c, '\t' | '\n' | ' '));
    Word
}
