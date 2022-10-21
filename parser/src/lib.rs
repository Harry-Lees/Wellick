/// Simple LL(1) Parser
/// Inspired by the Parser from the book "Crafting Interpreters"
/// https://craftinginterpreters.com/parsing-expressions.html#recursive-descent-parsing

mod ast;

use ast::{BinOp, ExprKind, Constant};

use lexer::{
    Token as LexToken,
    TokenKind as LexTokenKind
};

struct Parser {
    tokens: Vec<LexToken>,
    current: usize
}


impl Parser {
    fn new(tokens: Vec<LexToken>) -> Parser {
        Parser {
            tokens: tokens,
            current: 0
        }
    }

    /// Bump to the next token
    fn bump(&mut self) -> LexToken {
        // TODO: This function can panic if there is an array out of bounds error.
        // I think there's a way of not accessing array elements like this. This func
        // should probably return a Result or Option instead.
        let token = self.tokens[self.current];
        self.current += 1;
        token
    }

    /// Bump to the next token, ignoring whitespace
    fn bump_no_whitespace(&mut self) -> LexToken {
        let mut token = self.bump();
        while token.kind == LexTokenKind::Whitespace {
            token = self.bump();
        }
        token
    }

    /// Parse a single expression
    /// expression -> NUMBER OPERATOR NUMBER SEMICOLON
    fn statement(&mut self) -> Result<BinOp, String> {
        let left = match self.bump_no_whitespace() {
            LexToken {kind: LexTokenKind::Number(num), .. } => Constant::new(num),
            token @ _ => return Err(format!("SyntaxError line {}: expected number, got {:?}", token.line_no, token.kind))
        };
        let operator = match self.bump_no_whitespace() {
            LexToken {kind: LexTokenKind::Plus, .. } => ExprKind::Add,
            LexToken {kind: LexTokenKind::Minus, .. } => ExprKind::Sub,
            token @ _ => return Err(format!("SyntaxError line {}: expected operator, got {:?}", token.line_no, token.kind))
        };
        let right = match self.bump_no_whitespace() {
            LexToken {kind: LexTokenKind::Number(num), .. } => Constant::new(num),
            token @ _ => return Err(format!("SyntaxError line {}: expected number, got {:?}", token.line_no, token.kind))
        };

        let token = self.bump_no_whitespace();
        if token.kind != LexTokenKind::Semicolon {
            return Err(format!("SyntaxError line {}: expected semicolon", token.line_no));
        }

        Ok(BinOp::new(operator, left, right))
    }

    pub fn parse(&mut self) -> Result<BinOp, String> {
        self.statement()
    }
}

pub fn parse(tokens: Vec<LexToken>) {
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => println!("{:?}", ast),
        Err(err) => println!("{}", err)
    }
}