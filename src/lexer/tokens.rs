#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum TokenKind {
    Plus,
    Minus,
    Semicolon,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Number,
    Unknown,
    Whitespace,
    Newline,
    Eof
}
