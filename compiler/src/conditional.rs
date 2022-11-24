use crate::ast::{Atom, Compare, ComparisonOperator, Expression, Item};
use crate::expressions::expression;
use crate::helpers::{identifier_to_obj, ws};
use crate::literals::literal;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::{delimited, preceded, tuple};
use nom::IResult;

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

// comparison_operator = "==" | "!="
// comparison := identifier | literal comparison_operator identifier | literal;
pub fn comparison(input: &str) -> IResult<&str, Compare> {
    println!("parse comparison {:?}", input);

    // Parse the left side of the comparison
    let (i, left) = alt((
        map(ws(identifier_to_obj), |val| Atom::Name(val)),
        map(ws(literal), |val| Atom::Constant(val)),
    ))(input)?;

    // Parse the comparison operator
    let (i, op) = comparison_operator(i)?;

    // Parse the right side of the comparison
    let (i, right) = alt((
        map(ws(identifier_to_obj), |val| Atom::Name(val)),
        map(ws(literal), |val| Atom::Constant(val)),
    ))(i)?;

    Ok((
        i,
        Compare {
            left: left,
            op: op,
            right: right,
        },
    ))
}

pub fn if_stmt(input: &str) -> IResult<&str, (Compare, Vec<Expression>, Option<Vec<Expression>>)> {
    println!("parse if_stmt {:?}", input);
    map(
        tuple((
            preceded(tag("if"), ws(comparison)),
            delimited(tag("{"), many0(expression), tag("}")),
        )),
        |(comparison, if_then)| (comparison, if_then, Some(vec![])),
    )(input)
}
