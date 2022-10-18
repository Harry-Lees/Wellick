mod cursor;
pub use cursor::Cursor;

mod tokens;
pub use tokens::TokenKind;

pub fn is_newline(c: char) -> bool {
    matches!(
        c, '\u{000A}'..='\u{000D}' // Line Feed, Line Tabulation, Form Feed, Carriage Return
    )
}

/// True if `c` is considered a whitespace according to Rust language definition.
/// See [Rust language reference](https://doc.rust-lang.org/reference/whitespace.html)
/// for definitions of these classes.
pub fn is_whitespace(c: char) -> bool {
    // This is Pattern_White_Space.
    //
    // Note that this set is stable (ie, it doesn't change with different
    // Unicode versions), so it's ok to just hard-code the values.

    matches!(
        c,
        // Usual ASCII suspects
        '\u{0009}'   // \t
        | '\u{000B}' // vertical tab
        | '\u{0020}' // space

        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
    )
}


#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub line_no: u32,
    pub col_no: u32,
}

impl Token {
    pub fn new(kind: TokenKind, line_no: u32, col_no: u32) -> Token {
        Token { kind, line_no, col_no }
    }
}

impl Cursor<'_> {
    pub fn advance_token(&mut self) -> Token {
        let start = self.position();
        let first_char = match self.bump() {
            Some(c) => c,
            None => return Token::new(TokenKind::Eof, 1, 1)
        };

        let token_kind = match first_char {
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            ';' => TokenKind::Semicolon,
            '(' => TokenKind::OpenParen,
            ')' => TokenKind::CloseParen,
            '{' => TokenKind::OpenBrace,
            '}' => TokenKind::CloseBrace,
            c if is_whitespace(c) => {
                self.eat_while(is_whitespace);
                TokenKind::Whitespace
            },
            c if is_newline(c) => {
                self.eat_while(is_newline);
                self.advance_line();
                TokenKind::Newline
            },
            c @ '0'..='9' => {
                self.eat_decimal_digits();
                TokenKind::Number
            },
            c if first_char == '#' => {
                self.line_comment();
                TokenKind::Whitespace
            },
            _ => TokenKind::Unknown
        };
        let res = Token::new(
            token_kind,
            start.0,
            start.1,
        );
        res
    }

    fn eat_decimal_digits(&mut self) {
        loop {
            match self.first() {
                '_' => self.bump(),
                '0'..='9' => self.bump(),
                _ => break,
            };
        }
    }

    /// Eats the identifier.
    // fn eat_identifier(&mut self) -> Option<String> {
    //     if !is_id_start(self.first()) {
    //         return None;
    //     }
    //     self.bump();
    //     return Some(self.eat_while(is_id_continue));
    // }

    /// Eats a line comment.
    fn line_comment(&mut self) {
        self.bump();
        self.eat_while(|c| c != '\n');
    }
}

// Creates an iterator that produces tokens from the input string.
pub fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    let mut cursor = Cursor::new(input);
    std::iter::from_fn(move || {
        let token = cursor.advance_token();
        if token.kind != TokenKind::Eof {
            Some(token)
        } else {
            None
        }
    })
}
