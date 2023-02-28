mod compiler;
mod parser;

use nom::{multi::many0, IResult};
use parser::ast::*;
use parser::stmts::function;
use std::fs;

fn parse(input: &str) -> Result<Vec<FnDecl>, String> {
    let parser = many0(function)(input);
    match parser {
        IResult::Ok((remaining, result)) => {
            if remaining.len() > 0 {
                let msg = format!("failed to parse, unparsed tokens in file: {remaining}");
                return Err(msg);
            }
            println!("Successfully parsed program");
            return Ok(result);
        }
        _ => Err("failed to parse, an unknown error occurred".to_owned()),
    }
}

fn main() {
    let contents = fs::read_to_string("./examples/source.wellick").expect("unable to read file");
    let ast = match parse(contents.as_str()) {
        Ok(ast) => ast,
        Err(err) => {
            println!("{}", err);
            return ();
        }
    };

    dbg!(&ast);

    let aot_compiler = compiler::Compiler::default();

    match aot_compiler.compile(ast) {
        Ok(_) => println!("successfully compiled ast..."),
        Err(_) => println!("Failed to compile AST"),
    }
}
