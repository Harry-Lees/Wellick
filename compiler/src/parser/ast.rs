use std::option::Option;

/// Types supported by the language which also hold the corresponding
/// value, used in AST constructs like assignments
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Value {
    Float(f32),
    Integer(isize),
}

#[derive(Debug, Clone)]
pub enum EmptyType {
    Float,
    Integer,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub target: Name,
    pub value: Value,
}

impl Assignment {
    pub fn new(target: Name, value: Value) -> Self {
        Self { target, value }
    }
}

#[derive(Debug, Clone)]
pub struct FnArg {
    pub name: String,
    pub t: EmptyType,
}

impl FnArg {
    pub fn new(name: String, t: EmptyType) -> Self {
        Self { name, t }
    }
}

/// Function declaration AST node
#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub args: Vec<FnArg>,
    pub ret_type: Option<EmptyType>,
    pub body: Vec<Stmt>,
}

impl FnDecl {
    pub fn new(
        name: String,
        args: Vec<FnArg>,
        ret_type: Option<EmptyType>,
        body: Vec<Stmt>,
    ) -> Self {
        Self {
            name,
            args,
            ret_type,
            body,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub func: String,
    pub args: Vec<Name>,
}

#[derive(Debug, Clone)]
pub struct Name {
    pub ident: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ComparisonOperator {
    Eq,
    Gt,
    Lt,
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub value: String,
    pub _type: String,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Call(Call),
    Comparison(Box<Expression>, ComparisonOperator, Box<Expression>),
    Literal(Value),
    Identifier(String),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Return(Expression),
    If(Expression, Vec<Stmt>, Option<Vec<Stmt>>),
    Assign(Assignment),
}
