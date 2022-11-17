extern crate nom;
use nom::IResult;
use nom::bytes::complete::take_while;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::character::complete::one_of;
use nom::multi::many0;
use nom::multi::many1;
use nom::multi::separated_list0;
use nom::multi::separated_list1;
use nom::combinator::recognize;
use nom::branch::alt;
use nom::sequence::preceded;
use nom::sequence::separated_pair;
use nom::sequence::terminated;
use nom::sequence::tuple;
use nom::combinator::map_res;
use nom::combinator::map;

#[derive(Debug)]
struct Assignment {
    target: String,
    value: String,
}

#[derive(Debug)]
struct Call {
    func: String,
    args: Vec<String>,
}

impl Call {
    fn new(func: String, args: Vec<String>) -> Call {
        Call { func, args }
    }
}

impl Assignment {
    fn new(target: String, value: String) -> Assignment {
        Assignment { target, value }
    }
}

#[derive(Debug)]
enum ParseResult {
    Assign(Assignment),
    Call(Call)
}


fn from_call(input: (&str, &str, Vec<&str>, &str)) -> Result<ParseResult, String> {
    Ok(ParseResult::Call(Call::new(input.0.to_string(), vec!["abc".to_string(), "def".to_string()])))
}

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    // let func = | input: Vec<&str> | { String::from_iter(input) };
    println!("parse identifier {:?}", input);
    // take_while(| c: char | { c.is_ascii() && !(c == ',') && !(c == ')') })(input)
    map(
        many0(one_of("_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")),
        | input: Vec<char> | { String::from_iter(input).as_str() }
    )(input)
}


fn parse_function_call(input: &str) -> IResult<&str, ParseResult> {
    print!("parse function call {:?}", input);
    map_res(
        tuple((
            take_while(is_assignment_target),
            tag("("),
            separated_list0(tag(","), parse_identifier),
            tag(")"),
        )),
        from_call
    )(input)
    // map_res(
    //     tuple((
    //         take_while(is_assignment_target),
    //         tag("("),
    //         separated_list1(tag(","), is_assignment_target),
    //         tag(")")
    //     )),
    //     from_call
    // )(input)
}

fn is_assignment_target(c: char) -> bool {
    println!("is assignment target {:?}", c);
    c.is_alphabetic() & !(c == '=')
}


fn from_assignment(input: (&str, &str)) -> ParseResult {
    print!("{:?}", input);
    ParseResult::Assign(Assignment::new(
        input.0.to_owned(),
        input.1.to_owned()
    ))
}


fn parse_literal(input: &str) -> IResult<&str, &str> {
    alt((
        // hexadecimal literal
        preceded(tag("0x"), take_while(| c: char | { c.is_digit(16) })),
        // base 10 Numeric literal
        take_while(| c: char | { c.is_digit(10) }),
    ))(input)
}


fn parse_target(input: &str) -> IResult<&str, &str> {
    take_while(is_assignment_target)(input)
}


fn parse_assignment(input: &str) -> IResult<&str, ParseResult> {
    map(
        separated_pair(parse_target, char('='), parse_literal),
        from_assignment
    )(input)
}

fn parse_expression(input: &str) -> IResult<&str, ParseResult> {
    alt((
        terminated(parse_assignment, char(';')),
        terminated(parse_function_call, char(';')),
    ))(input)
}

fn parse(input: &str) -> Result<Vec<ParseResult>, &str> {
    match many0(parse_expression)(input) {
        IResult::Ok((remaining, result)) => {
            if remaining.len() > 0 {
                return Err("failed to parse, unparsed tokens in file")
            }
            Ok(result)
        },
        _ => Err("failed to parse, an unknown error occurred")
    }
}

fn main() {
    match parse("x=0x10;y=20;print();") {
        Ok(ast) => {
            println!("{:?}", ast);
        },
        Err(err) => {
            println!("{}", err);
        }
    }
}