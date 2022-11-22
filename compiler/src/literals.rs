use crate::helpers::ws;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{anychar, char, one_of};
use nom::combinator::{opt, recognize};
use nom::multi::{many0, many1, separated_list0};
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;

fn hexadecimal(input: &str) -> IResult<&str, &str> {
    preceded(
        tag_no_case("0x"),
        recognize(many1(terminated(
            one_of("0123456789abcdefABCDEF"),
            many0(char('_')),
        ))),
    )(input)
}

fn octal(input: &str) -> IResult<&str, &str> {
    preceded(
        tag_no_case("0o"),
        recognize(many1(terminated(one_of("01234567"), many0(char('_'))))),
    )(input)
}

fn decimal(input: &str) -> IResult<&str, &str> {
    recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(input)
}

fn float(input: &str) -> IResult<&str, &str> {
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
    ))(input)
}

fn array(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        tag("["),
        separated_list0(tag(","), ws(parse_literal)),
        tag("]"),
    )))(input)
}

pub fn parse_literal(input: &str) -> IResult<&str, &str> {
    alt((hexadecimal, octal, float, decimal, array))(input)
}
