use nom::branch::alt;
use nom::bytes::complete::tag_no_case;
use nom::character::complete::{char, one_of};
use nom::combinator::{map, opt, recognize};
use nom::multi::{many0, many1};
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;

use super::ast::{FloatLiteral, IntegerLiteral, Literal};

fn hexadecimal(input: &str) -> IResult<&str, Literal> {
    map(
        preceded(
            tag_no_case("0x"),
            recognize(many1(terminated(
                one_of("0123456789abcdefABCDEF"),
                many0(char('_')),
            ))),
        ),
        |value: &str| Literal::Integer(IntegerLiteral::new(value)),
    )(input)
}

fn octal(input: &str) -> IResult<&str, Literal> {
    map(
        preceded(
            tag_no_case("0o"),
            recognize(many1(terminated(one_of("01234567"), many0(char('_'))))),
        ),
        |value: &str| Literal::Integer(IntegerLiteral::new(value)),
    )(input)
}

fn decimal(input: &str) -> IResult<&str, Literal> {
    map(
        recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))),
        |value: &str| Literal::Integer(IntegerLiteral::new(value)),
    )(input)
}

fn float(input: &str) -> IResult<&str, Literal> {
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
        |value: &str| Literal::Float(FloatLiteral::new(value)),
    )(input)
}

pub fn literal(input: &str) -> IResult<&str, Literal> {
    alt((float, hexadecimal, octal, decimal))(input)
}
