use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::space1;
use nom::combinator::{map, opt};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, separated_pair, terminated};
use nom::sequence::{preceded, tuple};
use nom::IResult;

use super::ast::{EmptyType, FnArg, FnDecl};
use super::expressions::expression;
use super::helpers::{identifier, ws};

pub fn arg_type(input: &str) -> IResult<&str, EmptyType> {
    map(alt((tag("float"), tag("int"))), |val| match val {
        "float" => EmptyType::Float,
        "int" => EmptyType::Integer,
        _ => panic!(),
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
            opt(preceded(ws(tag("->")), arg_type)),
            delimited(ws(tag("{")), many0(expression), ws(tag("}"))),
        )),
        |(fn_name, fn_args, ret_type, body)| {
            FnDecl::new(fn_name.to_string(), fn_args, ret_type, body)
        },
    )(input)
}
