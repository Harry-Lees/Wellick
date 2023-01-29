mod compiler;
mod parser;

use nom::{multi::many0, IResult};
use parser::ast::*;
use parser::expressions::*;
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
    let ast = match parse(contents.as_str()) {
        Ok(ast) => ast,
        Err(err) => {
            println!("{}", err);
            return ();
        }
    };

    let aot_compiler = compiler::functions::Compiler::new();
    match aot_compiler.compile(ast) {
        Ok(_) => println!("successfully compiled ast..."),
        Err(_) => println!("Failed to compile AST"),
    }
}
