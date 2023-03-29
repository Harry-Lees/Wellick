use super::ast::{Call, Expression, Name};
use super::helpers::{identifier, identifier_to_obj, ws};
use super::literals::literal;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, multispace0};
use nom::combinator::{map, opt, value};
use nom::error::ParseError;
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

pub fn func_call(input: &str) -> IResult<&str, Call> {
    println!("In func_call {input:?}");
    map(
        tuple((
            // The function name
            identifier,
            // The function arguments, a list of identifiers or literals separated
            // by commas with any optional whitespace (including newlines).
            delimited(
                ws(tag("(")),
                terminated(
                    separated_list0(ws(tag(",")), expression),
                    // The function arguments may be terminated by an optional comma.
                    opt(tag(",")),
                ),
                ws(tag(")")),
            ),
        )),
        |(func, args)| Call {
            func: func.to_string(),
            args,
        },
    )(input)
}

pub fn reference(input: &str) -> IResult<&str, Name> {
    preceded(char('&'), identifier_to_obj)(input)
}

pub fn dereference(input: &str) -> IResult<&str, Name> {
    preceded(char('*'), identifier_to_obj)(input)
}

pub fn expression(input: &str) -> IResult<&str, Expression> {
    delimited(
        multispace0,
        alt((
            map(literal, |x| Expression::Literal(x)),
            map(func_call, |x| Expression::Call(x)),
            map(reference, |x| Expression::AddressOf(x.ident)),
            map(dereference, |x| Expression::DeRef(x.ident)),
            map(identifier_to_obj, |x| Expression::Identifier(x.ident)),
        )),
        multispace0,
    )(input)
}
