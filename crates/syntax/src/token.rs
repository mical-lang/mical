#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub len: u32,
}

const _: () = {
    assert!(size_of::<Token>() == size_of::<u64>());
    assert!(size_of::<Token>() == size_of::<Option<Token>>());
};

pub trait TokenStream<'src>: Iterator<Item = Token> {
    fn source(&self) -> &'src str;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Word,
    Numeral { radix: Radix, is_empty: bool },
    True,
    False,

    Tab,     // 0x09
    Newline, // 0x0A
    Space,   // 0x20

    CloseBrace, // }
    Greater,    // >
    Minus,      // -
    OpenBrace,  // {
    Pipe,       // |
    Plus,       // +
    Sharp,      // #
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Radix {
    Binary = 2,
    Octal = 8,
    Decimal = 10,
    Hexadecimal = 16,
}
