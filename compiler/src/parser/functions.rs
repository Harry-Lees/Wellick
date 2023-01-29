use crate::parser::ast::{FnArg, FnDecl, Type};
use crate::parser::expressions::expression;
use crate::parser::helpers::{identifier, ws};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::space1;
use nom::combinator::{map, opt};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, separated_pair, terminated};
use nom::sequence::{preceded, tuple};
use nom::IResult;

fn arg_type(input: &str) -> IResult<&str, Type> {
    map(alt((tag("f32"), tag("f64"), tag("isize"))), |t| match t {
        "isize" => Type::isize,
        "f32" => Type::f32,
        "f64" => Type::f64,
        _ => panic!("unexpected type found while parsing"),
    })(input)
}

fn function_args(input: &str) -> IResult<&str, Vec<FnArg>> {
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
    map(
        tuple((
            preceded(terminated(tag("fn"), space1), identifier),
            delimited(ws(tag("(")), function_args, ws(tag(")"))),
            delimited(ws(tag("{")), many0(expression), ws(tag("}"))),
        )),
        |(fn_name, fn_args, body)| FnDecl::new(fn_name.to_string(), fn_args, body),
    )(input)
}
