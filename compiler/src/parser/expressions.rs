use super::ast::{Call, ComparisonOperator, Expression};
use super::helpers::{identifier, identifier_to_obj, ws};
use super::literals::literal;

use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::{char, multispace0};
use nom::combinator::{map, opt};
use nom::error::{context, Error};
use nom::multi::separated_list0;
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;

// pub fn peol_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
//     value(
//         (), // Output is thrown away.
//         pair(char('%'), is_not("\n\r")),
//     )(i)
// }

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
                    separated_list0(ws(tag(",")), identifier_to_obj),
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

fn eq(input: &str) -> IResult<&str, ComparisonOperator> {
    let (i, _) = tag("==")(input)?;
    Ok((i, ComparisonOperator::Eq))
}

fn not_eq(input: &str) -> IResult<&str, ComparisonOperator> {
    let (i, _) = tag("!=")(input)?;
    Ok((i, ComparisonOperator::Eq))
}

fn lt(input: &str) -> IResult<&str, ComparisonOperator> {
    let (i, _) = tag("<")(input)?;
    Ok((i, ComparisonOperator::Lt))
}

fn gt(input: &str) -> IResult<&str, ComparisonOperator> {
    let (i, _) = tag(">")(input)?;
    Ok((i, ComparisonOperator::Gt))
}

fn comparison_operator(input: &str) -> IResult<&str, ComparisonOperator> {
    alt((lt, gt, eq, not_eq))(input)
}

/// comparison_operator = "==" | "!="
/// comparison := identifier | literal comparison_operator identifier | literal;
pub fn comparison(
    input: &str,
) -> IResult<&str, (Box<Expression>, ComparisonOperator, Box<Expression>)> {
    println!("in comparison {input:?}");
    // Parse the left side of the comparison
    let (i, left) = expression(input)?;

    // Parse the comparison operator
    let (i, op) = comparison_operator(i)?;

    // Parse the right side of the comparison
    let (i, right) = expression(i)?;

    Ok((i, (Box::from(left), op, Box::from(right))))
}

pub fn expression(input: &str) -> IResult<&str, Expression> {
    println!("In expression {input:?}");
    let (remaining, expr) = delimited(
        multispace0,
        alt((
            map(literal, |x| Expression::Literal(x)),
            map(func_call, |x| Expression::Call(x)),
            map(comparison, |(left, op, right)| {
                Expression::Comparison(left, op, right)
            }),
            map(identifier_to_obj, |x| Expression::Identifier(x.ident)),
        )),
        multispace0,
    )(input)?;

    context("Missing semicolon", char(';'))(remaining)?;

    Ok((remaining, expr))
}
