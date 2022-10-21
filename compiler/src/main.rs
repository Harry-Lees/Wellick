use std::env;
use std::io::Read;
use std::fs::File;

use lexer::tokenize;
use parser::parse;

fn main() -> Result<(), String> {
    let filename = get_file_name()?;
    let source = read_source_file(filename.as_str())?;

    let tokens = tokenize(source.as_str());
    parse(Vec::from_iter(tokens));

    Ok(())
}

fn get_file_name() -> Result<String, String> {
    let mut args = env::args();
    args.next(); // skip program name

    return args.next().ok_or(String::from("No filename provided"));
}

fn read_source_file(filename: &str) -> Result<String, String> {
    let mut file = File::open(filename).or(Err("Could not open file"))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).or(Err("Could not read file"))?;

    return Ok(contents);
}
