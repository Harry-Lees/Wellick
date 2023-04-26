use super::ast::Name;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, multispace0};
use nom::combinator::{opt, recognize};
use nom::error::ParseError;
use nom::multi::many0_count;
use nom::sequence::delimited;
use nom::sequence::pair;
use nom::IResult;

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

pub fn mutable_qualifier(input: &str) -> IResult<&str, bool> {
    let (i, result) = opt(tag("mut"))(input)?;
    if result.is_some() {
        return Ok((i, true));
    }
    return Ok((i, false));
}

/// From the nom [docs](https://github.com/rust-bakery/nom/blob/main/doc/nom_recipes.md#rust-style-identifiers)
/// A parser that mimics Rust style identifiers.
pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

/// From the nom [docs](https://github.com/rust-bakery/nom/blob/main/doc/nom_recipes.md#rust-style-identifiers)
/// A parser that mimics Rust style identifiers and returns a Name object.
pub fn identifier_to_obj(input: &str) -> IResult<&str, Name> {
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
