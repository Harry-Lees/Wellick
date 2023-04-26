use super::ast::{Assignment, EmptyType, Expression, FloatType, IntegerType, Local, Pointer, Stmt};
use super::ast::{FnArg, FnDecl};
use super::expressions::{expression, func_call};
use super::helpers::{arg_type, identifier, identifier_to_obj, ws};

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, space1};
use nom::combinator::{map, opt};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, preceded, separated_pair, terminated, tuple};
use nom::IResult;

pub fn assign_type(input: &str) -> IResult<&str, EmptyType> {
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
        map(
            tuple((preceded(ws(tag("*")), ws(mutable_qualifier)), arg_type)),
            |(mutable, val)| EmptyType::Pointer(Box::new(Pointer::new(val, mutable))),
        ),
    ))(input)
}

pub fn if_stmt(input: &str) -> IResult<&str, (Expression, Vec<Stmt>, Option<Vec<Stmt>>)> {
    map(
        tuple((
            preceded(tag("if"), ws(expression)),
            delimited(ws(tag("{")), many0(stmt), ws(tag("}"))),
        )),
        |(comparison, if_then)| (comparison, if_then, Some(vec![])),
    )(input)
}

fn function_args(input: &str) -> IResult<&str, Vec<FnArg>> {
    map(
        terminated(
            separated_list0(
                ws(tag(",")),
                separated_pair(identifier, ws(tag(":")), arg_type),
            ),
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
            delimited(ws(tag("{")), many0(stmt), ws(tag("}"))),
        )),
        |(fn_name, fn_args, ret_type, body)| {
            FnDecl::new(fn_name.to_string(), fn_args, ret_type, body)
        },
    )(input)
}

pub fn mutable_qualifier(input: &str) -> IResult<&str, bool> {
    let (i, result) = opt(tag("mut"))(input)?;
    if result.is_some() {
        return Ok((i, true));
    }
    return Ok((i, false));
}

/// Parse assignment in the form
/// let <var_name>: <var_type> = <value>
/// e.g. let x: f32 = 10.0;
pub fn assignment(input: &str) -> IResult<&str, Assignment> {
    map(
        preceded(
            ws(tag("let")),
            tuple((
                ws(mutable_qualifier),
                identifier_to_obj,
                preceded(ws(tag(":")), assign_type),
                ws(char('=')),
                expression,
            )),
        ),
        |(mutable, target, var_type, _, value)| Assignment::new(target, var_type, value, mutable),
    )(input)
}

/// Parse a re-assignment.
/// This is when an already defined variable is changed.
pub fn reassign(input: &str) -> IResult<&str, Local> {
    map(
        tuple((identifier_to_obj, ws(char('=')), expression)),
        |(target, _, value)| Local::new(target, value),
    )(input)
}

pub fn return_(input: &str) -> IResult<&str, Expression> {
    preceded(ws(tag("return")), expression)(input)
}

pub fn stmt(input: &str) -> IResult<&str, Stmt> {
    alt((
        map(if_stmt, |(comparison, body, else_body)| {
            Stmt::If(comparison, body, else_body)
        }),
        map(terminated(return_, ws(char(';'))), |x| Stmt::Return(x)),
        map(terminated(func_call, ws(char(';'))), |x| Stmt::Call(x)),
        map(terminated(reassign, ws(char(';'))), |x| Stmt::ReAssign(x)),
        map(terminated(assignment, ws(char(';'))), |x| Stmt::Assign(x)),
    ))(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::ast;
    use crate::parser::stmts::{assignment, function, reassign};

    #[test]
    fn test_parse_reassign() {
        let code = "x = 10";
        reassign(code).unwrap();
    }

    #[test]
    fn test_parse_empty_decl() {
        let code = "fn main() {}";
        function(code).unwrap();
    }

    #[test]
    fn test_parse_decl_with_args() {
        let declarations = [
            "fn main(x: i32) {}",
            "fn main(x: i32, y: i32) {}",
            "fn main(x: i32, y: i64) {}",
            "fn main(x: i32, y: i32, z: i32) {}",
        ];

        for declaration in declarations {
            function(declaration).unwrap();
        }
    }

    #[test]
    fn test_parse_assignment() {
        let assignments = [
            "let x: i32 = 10;",
            "let mut x: i32 = 10;",
            "let mut y: i32 = &x;",
            "let mut y: *i32 = &x;",
            "let mut y: *mut i32 = &x;",
        ];

        for assign in assignments {
            assignment(assign).unwrap();
        }
    }

    #[test]
    fn test_mutable_ptr() {
        let (_, ast) = assignment("let mut y: *mut i32 = &x").unwrap();
        assert!(ast.mutable);
        match ast.var_type {
            ast::EmptyType::Pointer(ptr) => {
                assert!(ptr.mutable);
            }
            _ => {
                unreachable!("ptr assignment parsed as incorrect type");
            }
        };
    }
}
