use crate::parser::ast::Constant;
use crate::parser::helpers::ws;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{char, one_of};
use nom::combinator::{map, opt, recognize};
use nom::multi::{many0, many1, separated_list0};
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;

fn hexadecimal(input: &str) -> IResult<&str, Constant> {
    map(
        preceded(
            tag_no_case("0x"),
            recognize(many1(terminated(
                one_of("0123456789abcdefABCDEF"),
                many0(char('_')),
            ))),
        ),
        |value: &str| Constant {
            value: value.to_string(),
            _type: "u32".to_string(),
        },
    )(input)
}

fn octal(input: &str) -> IResult<&str, Constant> {
    map(
        preceded(
            tag_no_case("0o"),
            recognize(many1(terminated(one_of("01234567"), many0(char('_'))))),
        ),
        |value: &str| Constant {
            value: value.to_string(),
            _type: "u32".to_string(),
        },
    )(input)
}

fn array(input: &str) -> IResult<&str, Constant> {
    map(
        recognize(tuple((
            tag("["),
            separated_list0(tag(","), ws(literal)),
            tag("]"),
        ))),
        |value| Constant {
            value: value.to_string(),
            _type: "array".to_string(),
        },
    )(input)
}

fn decimal(input: &str) -> IResult<&str, Constant> {
    map(
        recognize(many1(terminated(one_of("0123456789"), many0(char('_'))))),
        |value: &str| Constant {
            value: value.to_string(),
            _type: "u32".to_string(),
        },
    )(input)
}

fn float(input: &str) -> IResult<&str, Constant> {
    let (i, parsed) = alt((
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
    ))(input)?;

    Ok((
        i,
        Constant {
            value: parsed.to_string(),
            _type: "f32".to_string(),
        },
    ))
}

pub fn literal(input: &str) -> IResult<&str, Constant> {
    dbg!("literal {:?}", input);
    alt((float, hexadecimal, octal, array, decimal))(input)
}
