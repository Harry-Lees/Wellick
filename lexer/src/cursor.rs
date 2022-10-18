use std::str::Chars;

/// Peekable iterator over a char sequence.
///
/// Next characters can be peeked via `first` method,
/// and position can be shifted forward via `bump` method.
pub struct Cursor<'a> {
    /// Iterator over chars. Slightly faster than a &str.
    chars: Chars<'a>,
    line_no: u32,
    col_no: u32,
}

pub(crate) const EOF_CHAR: char = '\0';

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Cursor<'a> {
        Cursor {
            chars: input.chars(),
            line_no: 1,
            col_no: 1,
        }
    }

    /// Peeks the next symbol from the input stream without consuming it.
    /// If requested position doesn't exist, `EOF_CHAR` is returned.
    /// However, getting `EOF_CHAR` doesn't always mean actual end of file,
    /// it should be checked with `is_eof` method.
    pub(crate) fn first(&self) -> char {
        // `.next()` optimizes better than `.nth(0)`
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    /// Checks if there is nothing more to consume.
    pub(crate) fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    /// Advances the cursor to the next line in the file.
    pub(crate) fn advance_line(&mut self) {
        self.line_no += 1;
        self.col_no = 1;
    }

    /// Gets the current cursor position.
    /// Position is returned as a 2-tuple of
    /// line number, column number.
    pub(crate) fn position(&self) -> (u32, u32) {
        (self.line_no, self.col_no)
    }

    /// Moves to the next character.
    pub(crate) fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.col_no += 1;
        Some(c)
    }

    /// Eats symbols while predicate returns true or until the end of file is reached.
    pub(crate) fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) -> String {
        let mut vec: Vec<char> = Vec::new();
        while predicate(self.first()) && !self.is_eof() {
            if let Some(c) = self.bump() {
                vec.push(c);
            }
        }
        String::from_iter(vec)
    }
}