use super::ast::{Assignment, Atom, Call, Expression, Item};
use super::conditional::if_stmt;
use super::functions::arg_type;
use super::functions::function;
use super::helpers::{identifier, identifier_to_obj, ws};
use super::literals::literal;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, multispace0};
use nom::combinator::{map, opt};
use nom::multi::separated_list0;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::sequence::{delimited, tuple};
use nom::IResult;

// pub fn peol_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
//     value(
//         (), // Output is thrown away.
//         pair(char('%'), is_not("\n\r")),
//     )(i)
// }

/// Parse an Item
pub fn item(input: &str) -> IResult<&str, Item> {
    map(function, |x| Item::FnDecl(x))(input)
}

pub fn func_call(input: &str) -> IResult<&str, Call> {
    dbg!("parse function call {:?}", input);

    map(
        tuple((
            // The function name
            identifier,
            // The function arguments, a list of identifiers or literals separated
            // by commas with any optional whitespace (including newlines).
            delimited(
                ws(tag("(")),
                terminated(
                    separated_list0(
                        ws(tag(",")),
                        alt((
                            map(identifier_to_obj, |x| Atom::Name(x)),
                            map(literal, |x| Atom::Constant(x)),
                        )),
                    ),
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

/// Parse assignment in the form
/// let <var_name>: <var_type> = <value>
/// e.g. let x: f32 = 10.0;
pub fn assignment(input: &str) -> IResult<&str, Assignment> {
    dbg!("parse_assignment {:?}", input);

    map(
        tuple((
            identifier_to_obj,
            opt(preceded(ws(tag(":")), arg_type)),
            ws(char('=')),
            literal,
        )),
        |(target, _, _, value)| Assignment::new(target, value),
    )(input)
}

pub fn expression(input: &str) -> IResult<&str, Expression> {
    dbg!("parse expression {:?}", input);
    delimited(
        multispace0,
        alt((
            map(terminated(assignment, char(';')), |x| Expression::Assign(x)),
            map(terminated(func_call, char(';')), |x| Expression::Call(x)),
            map(if_stmt, |(cond, if_then, else_then)| {
                Expression::If(cond, if_then, else_then)
            }),
        )),
        multispace0,
    )(input)
}
