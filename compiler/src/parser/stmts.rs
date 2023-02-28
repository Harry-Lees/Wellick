use super::ast::{Assignment, Expression, Stmt};
use super::ast::{FnArg, FnDecl};
use super::expressions::expression;
use super::helpers::{arg_type, identifier, identifier_to_obj, ws};
use super::literals::literal;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, space1};
use nom::combinator::{map, opt};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom::IResult;

pub fn if_stmt(input: &str) -> IResult<&str, (Expression, Vec<Stmt>, Option<Vec<Stmt>>)> {
    println!("In if_stmt");
    map(
        tuple((
            preceded(tag("if"), ws(expression)),
            delimited(ws(tag("{")), many0(stmt), ws(tag("}"))),
        )),
        |(comparison, if_then)| (comparison, if_then, Some(vec![])),
    )(input)
}

fn function_args(input: &str) -> IResult<&str, Vec<FnArg>> {
    println!("In function args");
    map(
        terminated(
            separated_list0(tag(","), separated_pair(identifier, ws(tag(":")), arg_type)),
            opt(tag(",")),
        ),
        |args| {
            args.into_iter()
                .map(|(arg, t)| FnArg::new(arg.to_string(), t))
                .collect()
        },
    )(input)
}

pub fn function(input: &str) -> IResult<&str, FnDecl> {
    println!("Parsing func");
    map(
        tuple((
            preceded(terminated(tag("fn"), space1), identifier),
            delimited(ws(tag("(")), function_args, ws(tag(")"))),
            opt(preceded(ws(tag("->")), arg_type)),
            delimited(ws(tag("{")), many0(stmt), ws(tag("}"))),
        )),
        |(fn_name, fn_args, ret_type, body)| {
            println!("Successfully parsed {fn_name}");
            FnDecl::new(fn_name.to_string(), fn_args, ret_type, body)
        },
    )(input)
}

/// Parse assignment in the form
/// let <var_name>: <var_type> = <value>
/// e.g. let x: f32 = 10.0;
pub fn assignment(input: &str) -> IResult<&str, Assignment> {
    println!("In assignment");
    map(
        tuple((
            identifier_to_obj,
            preceded(ws(tag(":")), arg_type),
            ws(char('=')),
            expression,
        )),
        |(target, var_type, _, value)| Assignment::new(target, var_type, value),
    )(input)
}

pub fn return_(input: &str) -> IResult<&str, Expression> {
    preceded(ws(tag("return")), expression)(input)
}

pub fn stmt(input: &str) -> IResult<&str, Stmt> {
    println!("In stmt {input}");
    alt((
        map(if_stmt, |(comparison, body, else_body)| {
            Stmt::If(comparison, body, else_body)
        }),
        map(terminated(return_, ws(char(';'))), |x| {
            println!("Successfully parsed return {x:?}");
            Stmt::Return(x)
        }),
        map(terminated(assignment, ws(char(';'))), |x| Stmt::Assign(x)),
    ))(input)
}
