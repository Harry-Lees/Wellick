extern crate nom;
pub mod ast;
pub mod conditional;
pub mod expressions;
pub mod functions;
pub mod helpers;
pub mod literals;

use ast::*;
use expressions::*;
use nom::{multi::many0, IResult};
use std::fs;

fn parse(input: &str) -> Result<Vec<Item>, &str> {
    let parser = many0(item)(input);
    match parser {
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
            println!();
            for item in ast {
                println!("{:?}", item);
            }
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}
