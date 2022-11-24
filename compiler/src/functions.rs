use crate::ast::{FnDecl, Name};
use crate::expressions::expression;
use crate::helpers::{identifier, ws};

use nom::bytes::complete::tag;
use nom::character::complete::space1;
use nom::combinator::{map, opt};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, terminated};
use nom::sequence::{preceded, tuple};
use nom::IResult;

pub fn function(input: &str) -> IResult<&str, FnDecl> {
    map(
        tuple((
            preceded(terminated(tag("fn"), space1), identifier),
            delimited(
                ws(tag("(")),
                terminated(separated_list0(tag(","), ws(identifier)), opt(tag(","))),
                ws(tag(")")),
            ),
            delimited(ws(tag("{")), many0(expression), ws(tag("}"))),
        )),
        |(fn_name, fn_args, body)| FnDecl {
            name: fn_name.to_string(),
            args: fn_args
                .iter()
                .map(|x| Name {
                    ident: x.to_string(),
                })
                .collect(),
            body: body,
        },
    )(input)
}
