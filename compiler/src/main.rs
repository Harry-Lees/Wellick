extern crate nom;
pub mod ast;
pub mod helpers;
pub mod literals;

use ast::*;
use helpers::ws;
use literals::parse_literal;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{alpha1, alphanumeric1, char, multispace0};
use nom::combinator::{map, opt, recognize, value};
use nom::error::ParseError;
use nom::multi::{many0, many0_count, separated_list0};
use nom::sequence::delimited;
use nom::sequence::pair;
use nom::sequence::separated_pair;
use nom::sequence::terminated;
use nom::IResult;
use std::fs;

pub fn peol_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
    value(
        (), // Output is thrown away.
        pair(char('%'), is_not("\n\r")),
    )(i)
}

fn from_call(func: &str, args: Vec<&str>) -> ParseResult {
    let owned_args: Vec<String> = args.iter().map(|item| item.to_string()).collect();
    ParseResult::Call(Call::new(func.to_string(), owned_args))
}

fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn parse_function_call(input: &str) -> IResult<&str, ParseResult> {
    println!("parse function call {:?}", input);
    let (i, func) = identifier(input)?;
    let (i, _) = ws(tag("("))(i)?;
    let (i, args) = terminated(separated_list0(tag(","), ws(identifier)), opt(tag(",")))(i)?;
    let (i, _) = ws(tag(")"))(i)?;

    Ok((i, from_call(func, args)))
}

fn from_assignment(input: (&str, &str)) -> ParseResult {
    println!("from_assignment {:?}", input);
    ParseResult::Assign(Assignment::new(input.0.to_owned(), input.1.to_owned()))
}

fn parse_assignment(input: &str) -> IResult<&str, ParseResult> {
    map(
        separated_pair(identifier, ws(char('=')), parse_literal),
        from_assignment,
    )(input)
}

fn parse_expression(input: &str) -> IResult<&str, ParseResult> {
    delimited(
        multispace0,
        alt((
            terminated(parse_assignment, char(';')),
            terminated(parse_function_call, char(';')),
        )),
        multispace0,
    )(input)
}

fn parse(input: &str) -> Result<Vec<ParseResult>, &str> {
    match many0(parse_expression)(input) {
        IResult::Ok((remaining, result)) => {
            if remaining.len() > 0 {
                return Err("failed to parse, unparsed tokens in file");
            }
            Ok(result)
        }
        _ => Err("failed to parse, an unknown error occurred"),
    }
}

fn main() {
    let contents = fs::read_to_string("./examples/source.txt").expect("unable to read file");
    match parse(contents.as_str()) {
        Ok(ast) => {
            println!("{:?}", ast);
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}
