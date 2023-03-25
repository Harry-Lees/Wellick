use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{char, one_of};
use nom::combinator::{map, opt, recognize};
use nom::multi::{many0, many1};
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;

use super::ast::Value;

fn hexadecimal(input: &str) -> IResult<&str, Value> {
    map(
        preceded(
            tag_no_case("0x"),
            recognize(many1(terminated(
                one_of("0123456789abcdefABCDEF"),
                many0(char('_')),
            ))),
        ),
        |value: &str| Value::I32(value.parse::<i32>().unwrap()),
    )(input)
}

fn octal(input: &str) -> IResult<&str, Value> {
    map(
        preceded(
            tag_no_case("0o"),
            recognize(many1(terminated(one_of("01234567"), many0(char('_'))))),
        ),
        |value: &str| Value::I32(value.parse::<i32>().unwrap()),
    )(input)
}

fn decimal(input: &str) -> IResult<&str, Value> {
    map(
        recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))),
        |value: &str| Value::I32(value.parse::<i32>().unwrap()),
    )(input)
}

fn float(input: &str) -> IResult<&str, Value> {
    map(
        alt((
            // Case one: .42
            recognize(tuple((
                char('.'),
                decimal,
                opt(tuple((one_of("eE"), opt(one_of("+-")), decimal))),
            ))), // Case two: 42e42 and 42.42e42
            recognize(tuple((
                decimal,
                opt(preceded(char('.'), decimal)),
                one_of("eE"),
                opt(one_of("+-")),
                decimal,
            ))), // Case three: 42. and 42.42
            recognize(tuple((decimal, char('.'), opt(decimal)))),
        )),
        |value: &str| Value::F32(value.parse::<f32>().unwrap()),
    )(input)
}

pub fn literal(input: &str) -> IResult<&str, Value> {
    println!("in literal {input:?}");
    alt((float, hexadecimal, octal, decimal))(input)
}
