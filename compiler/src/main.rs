mod compiler;
mod parser;

use clap::Parser;
use nom::{multi::many0, IResult};
use parser::ast::*;
use parser::stmts::function;
use std::fs;

#[derive(Parser)]
struct Cli {
    path: std::path::PathBuf,
}

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

fn main() -> Result<(), String> {
    let args = Cli::parse();
    let contents = fs::read_to_string(args.path).expect("unable to read file");
    let ast = match parse(contents.as_str()) {
        Ok(ast) => {
            println!("Successfully constructed AST");
            ast
        }
        Err(err) => {
            println!("SyntaxError: {}", err);
            return Err("Failed to compile".to_owned());
        }
    };

    dbg!(&ast);

    let aot_compiler = compiler::Compiler::default();

    aot_compiler.compile(ast)?;

    Ok(())
}
