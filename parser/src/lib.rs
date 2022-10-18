/// Simple LL(1) Parser
/// Inspired by the Parser from the book "Crafting Interpreters"
/// https://craftinginterpreters.com/parsing-expressions.html#recursive-descent-parsing

use lexer::{
    Token as LexToken,
    TokenKind as LexTokenKind
};

#[derive(Debug, Clone)]
pub enum GrammarItem {
    Product,
    Sum,
    Number(u64),
    Paren
}

#[derive(Debug, Clone)]
pub struct ParseNode {
    pub children: Vec<ParseNode>,
    pub entry: GrammarItem,
}

impl ParseNode {
    pub fn new() -> ParseNode {
        ParseNode {
            children: Vec::new(),
            entry: GrammarItem::Paren,
        }
    }
}

fn get_number<T: Iterator<Item = char>>(c: char, iter: &mut Peekable<T>) -> u64 {
    let mut number = c.to_string().parse::<u64>().expect("The caller should have passed a digit.");
    while let Some(Ok(digit)) = iter.peek().map(|c| c.to_string().parse::<u64>()) {
        number = number * 10 + digit;
        iter.next();
    }
    number
}

pub fn parse(input: Vec<LexToken>) -> Result<ParseNode, String> {
    parse_expr(&tokens, 0).and_then(|(n, i)| if i == tokens.len() {
        Ok(n)
    } else {
        Err(format!("Expected end of input, found {:?} at {}", tokens[i], i))
    })
}

fn parse_expr(tokens: &Vec<LexToken>, pos: usize) -> Result<(ParseNode, usize), String> {
    let (node_summand, next_pos) = parse_summand(tokens, pos)?;
    let c = tokens.get(next_pos);
    match c {
        Some(&LexToken::Op('+')) => {
            // recurse on the expr
            let mut sum = ParseNode::new();
            sum.entry = GrammarItem::Sum;
            sum.children.push(node_summand);
            let (rhs, i) = parse_expr(tokens, next_pos + 1)?;
            sum.children.push(rhs);
            Ok((sum, i))
        }
        _ => {
            // we have just the summand production, nothing more.
            Ok((node_summand, next_pos))
        }
    }
}

fn parse_summand(tokens: &Vec<LexToken>, pos: usize) -> Result<(ParseNode, usize), String> {
    let (node_term, next_pos) = parse_term(tokens, pos)?;
    let c = tokens.get(next_pos);
    match c {
        Some(&LexToken::Op('*')) => {
            // recurse on the summand
            let mut product = ParseNode::new();
            product.entry = GrammarItem::Product;
            product.children.push(node_term);
            let (rhs, i) = parse_summand(tokens, next_pos + 1)?;
            product.children.push(rhs);
            Ok((product, i))
        }
        _ => {
            // we have just the term production, nothing more.
            Ok((node_term, next_pos))
        }
    }
}

fn parse_term(tokens: &Vec<LexToken>, pos: usize) -> Result<(ParseNode, usize), String> {
    let c: &LexToken = tokens.get(pos)
        .ok_or(String::from("Unexpected end of input, expected paren or number"))?;
    match c {
        &LexToken::Num(n) => {
            let mut node = ParseNode::new();
            node.entry = GrammarItem::Number(n);
            Ok((node, pos + 1))
        }
        &LexToken::Paren(c) => {
            match c {
                '(' | '[' | '{' => {
                    parse_expr(tokens, pos + 1).and_then(|(node, next_pos)| {
                        if let Some(&LexToken::Paren(c2)) = tokens.get(next_pos) {
                            if c2 == matching(c) {
                                // okay!
                                let mut paren = ParseNode::new();
                                paren.children.push(node);
                                Ok((paren, next_pos + 1))
                            } else {
                                Err(format!("Expected {} but found {} at {}",
                                            matching(c),
                                            c2,
                                            next_pos))
                            }
                        } else {
                            Err(format!("Expected closing paren at {} but found {:?}",
                                        next_pos,
                                        tokens.get(next_pos)))
                        }
                    })
                }
                _ => Err(format!("Expected paren at {} but found {:?}", pos, c)),
            }
        }
        _ => {
            Err(format!("Unexpected token {:?}, expected paren or number", {
                c
            }))
        }
    }
}

fn matching(c: char) -> char {
    match c {
        ')' => '(',
        ']' => '[',
        '}' => '{',
        '(' => ')',
        '[' => ']',
        '{' => '}',
        _ => panic!("should have been a parenthesis!"),
    }
}

pub fn print(tree: &ParseNode) -> String {
    match tree.entry {
        GrammarItem::Paren => {
            format!("({})",
                    print(tree.children.get(0).expect("parens need one child")))
        }
        GrammarItem::Sum => {
            let lhs = print(tree.children.get(0).expect("sums need two children"));
            let rhs = print(tree.children.get(1).expect("sums need two children"));
            format!("{} + {}", lhs, rhs)
        }
        GrammarItem::Product => {
            let lhs = print(tree.children.get(0).expect("products need two children"));
            let rhs = print(tree.children.get(1).expect("products need two children"));
            format!("{} * {}", lhs, rhs)
        }
        GrammarItem::Number(n) => format!("{}", n),
    }
}
