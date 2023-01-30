use std::option::Option;

// pub enum Keyword {
//     Pass,
//     Return,
// }

// Types supported by the language
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Type {
    isize,
    f32,
    f64,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub target: Name,
    pub var_type: Type,
    pub value: Atom,
}

impl Assignment {
    pub fn new(target: Name, var_type: Type, value: Atom) -> Self {
        Self {
            target,
            var_type,
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnArg {
    pub name: String,
    pub t: Type,
}

impl FnArg {
    pub fn new(name: String, t: Type) -> Self {
        Self { name, t }
    }
}

/// Function declaration AST node
#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub args: Vec<FnArg>,
    pub ret_type: Option<Type>,
    pub body: Vec<Expression>,
}

impl FnDecl {
    pub fn new(
        name: String,
        args: Vec<FnArg>,
        ret_type: Option<Type>,
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
    pub args: Vec<Atom>,
}

#[derive(Debug, Clone)]
pub struct Name {
    pub ident: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ComparisonOperator {
    Eq,
    NotEq,
    Gt,
    Lt,
}

#[derive(Debug, Clone)]
pub enum Atom {
    Name(Name),
    Constant(Constant),
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
pub enum Stmt {
    Item(Item),
    Expr(Expression),
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
}
