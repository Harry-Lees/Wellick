use crate::ast::{Assignment, Atom, Call, Expression, Item, Name};
use crate::conditional::if_stmt;
use crate::functions::function;
use crate::helpers::{identifier, identifier_to_obj, ws};
use crate::literals::literal;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::character::complete::{char, multispace0};
use nom::combinator::{map, opt, value};
use nom::error::ParseError;
use nom::multi::separated_list0;
use nom::sequence::pair;
use nom::sequence::separated_pair;
use nom::sequence::terminated;
use nom::sequence::{delimited, tuple};
use nom::IResult;

pub fn peol_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
    value(
        (), // Output is thrown away.
        pair(char('%'), is_not("\n\r")),
    )(i)
}

/// Parse an Item
pub fn item(input: &str) -> IResult<&str, Item> {
    map(function, |x| Item::FnDecl(x))(input)
}

pub fn func_call(input: &str) -> IResult<&str, Call> {
    println!("parse function call {:?}", input);

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

pub fn assignment(input: &str) -> IResult<&str, Assignment> {
    println!("parse_assignment {:?}", input);

    map(separated_pair(identifier, ws(char('=')), literal), |val| {
        Assignment {
            target: Name {
                ident: val.0.to_string(),
            },
            value: Atom::Constant(val.1),
        }
    })(input)
}

pub fn expression(input: &str) -> IResult<&str, Expression> {
    println!("parse expression {:?}", input);
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
