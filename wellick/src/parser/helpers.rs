use super::ast::{EmptyType, FloatType, IntegerType, Name, Pointer};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, multispace0};
use nom::combinator::{map, recognize};
use nom::error::ParseError;
use nom::multi::many0_count;
use nom::sequence::pair;
use nom::sequence::{delimited, preceded};
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

pub fn arg_type(input: &str) -> IResult<&str, EmptyType> {
    alt((
        map(
            alt((tag("f32"), tag("f64"), tag("i32"), tag("i64"), tag("isize"))),
            |val| match val {
                "f32" => EmptyType::Float(FloatType::F32),
                "f64" => EmptyType::Float(FloatType::F64),
                "i32" => EmptyType::Integer(IntegerType::I32),
                "i64" => EmptyType::Integer(IntegerType::I64),
                "isize" => EmptyType::Integer(IntegerType::PointerSize),
                _ => unreachable!(),
            },
        ),
        map(preceded(tag("*"), arg_type), |val| {
            EmptyType::Pointer(Box::new(Pointer::new(val, false)))
        }),
    ))(input)
}
