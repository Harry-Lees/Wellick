#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum TokenKind {
    Plus,
    Minus,
    Semicolon,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Number(i32),
    Unknown,
    Whitespace,
    Newline,
    Eof
}
