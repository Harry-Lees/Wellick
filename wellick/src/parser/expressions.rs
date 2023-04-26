use super::ast::{AddressOf, Call, Expression, Name};
use super::helpers::{identifier, identifier_to_obj, mutable_qualifier, ws};
use super::literals::literal;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, multispace0};
use nom::combinator::{map, opt};
use nom::multi::separated_list0;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

pub fn func_call(input: &str) -> IResult<&str, Call> {
    map(
        tuple((
            // The function name
            identifier,
            // The function arguments, a list of identifiers or literals separated
            // by commas with any optional whitespace (including newlines).
            delimited(
                ws(tag("(")),
                terminated(
                    separated_list0(ws(tag(",")), expression),
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

pub fn reference(input: &str) -> IResult<&str, AddressOf> {
    map(
        tuple((preceded(char('&'), ws(mutable_qualifier)), identifier)),
        |(mutable, identifier)| AddressOf::new(identifier.to_string(), mutable),
    )(input)
}

pub fn dereference(input: &str) -> IResult<&str, Name> {
    preceded(char('*'), identifier_to_obj)(input)
}

pub fn expression(input: &str) -> IResult<&str, Expression> {
    delimited(
        multispace0,
        alt((
            map(literal, |x| Expression::Literal(x)),
            map(func_call, |x| Expression::Call(x)),
            map(reference, |x| Expression::AddressOf(x)),
            map(dereference, |x| Expression::DeRef(x.ident)),
            map(identifier_to_obj, |x| Expression::Identifier(x.ident)),
        )),
        multispace0,
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::expressions::{dereference, reference};

    #[test]
    fn test_deref() -> Result<(), String> {
        let derefs = ["*x"];
        for deref in derefs {
            match dereference(deref) {
                Ok((remaining, _)) => {
                    assert_eq!(remaining.len(), 0);
                }
                Err(_) => {
                    return Err(format!("Failed to parse dereference {}", deref));
                }
            }
        }
        Ok(())
    }

    #[test]
    fn test_ref() -> Result<(), String> {
        let refs = ["&x", "&mut x"];

        for ref_ in refs {
            match reference(ref_) {
                Ok((remaining, _)) => {
                    assert_eq!(remaining.len(), 0);
                }
                Err(_) => {
                    return Err(format!("Failed to parse ref {}", ref_));
                }
            }
        }
        Ok(())
    }
}
