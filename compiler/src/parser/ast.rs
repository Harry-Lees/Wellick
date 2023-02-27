use std::option::Option;

#[derive(Debug, Clone)]
pub struct Return {
    pub value: Value,
}

impl Return {
    pub fn new(value: Value) -> Self {
        Self { value }
    }
}

impl Default for Return {
    fn default() -> Self {
        Self {
            value: Value::Integer(0),
        }
    }
}

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
    pub body: Vec<Expression>,
}

impl FnDecl {
    pub fn new(
        name: String,
        args: Vec<FnArg>,
        ret_type: Option<EmptyType>,
        body: Vec<Expression>,
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
pub enum Atom {
    Name(Name),
    Constant(Value),
}

#[derive(Debug, Clone)]
pub struct Compare {
    pub left: Atom,
    pub op: ComparisonOperator,
    pub right: Atom,
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub value: String,
    pub _type: String,
}

#[derive(Debug)]
pub enum Item {
    FnDecl(FnDecl),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Assign(Assignment),
    Call(Call),
    If(Compare, Vec<Expression>, Option<Vec<Expression>>),
    Return(Return),
}
