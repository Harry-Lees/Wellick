use crate::parser::ast::Name;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, multispace0};
use nom::combinator::{map, recognize};
use nom::error::ParseError;
use nom::multi::many0_count;
use nom::sequence::delimited;
use nom::sequence::pair;
use nom::IResult;

use super::ast::Type;

/// From the nom [docs](https://github.com/rust-bakery/nom/blob/main/doc/nom_recipes.md#wrapper-combinators-that-eat-whitespace-before-and-after-a-parser)
/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
pub(crate) fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

pub fn identifier_to_obj(input: &str) -> IResult<&str, Name> {
    dbg!("identifier_to_obj {:?}", input);
    let (i, ident) = recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)?;

    Ok((
        i,
        Name {
            ident: ident.to_string(),
        },
    ))
}

pub fn arg_type(input: &str) -> IResult<&str, Type> {
    map(alt((tag("f32"), tag("f64"), tag("isize"))), |t| match t {
        "isize" => Type::isize,
        "f32" => Type::f32,
        "f64" => Type::f64,
        _ => panic!("unexpected type found while parsing"),
    })(input)
}
