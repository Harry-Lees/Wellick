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
    Pointer,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub target: Name,
    pub value: Expression,
    pub var_type: Option<EmptyType>,
    pub mutable: bool,
}

impl Assignment {
    pub fn new(
        target: Name,
        var_type: Option<EmptyType>,
        value: Expression,
        mutable: bool,
    ) -> Self {
        Self {
            target,
            var_type,
            value,
            mutable,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Local {
    pub target: Name,
    pub value: Expression,
}

impl Local {
    pub fn new(target: Name, value: Expression) -> Self {
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
    pub args: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct Name {
    pub ident: String,
}

#[derive(Debug, Clone)]
pub struct Constant {
    pub value: String,
    pub _type: String,
}

#[derive(Debug, Clone)]
pub enum Expression {
    // A function call e.g. f();
    Call(Call),

    // A literal value e.g. 10.0
    Literal(Value),

    // A variable identifier e.g. x;
    Identifier(String),

    // Address-of a variable e.g. &x; gets the address of x.
    AddressOf(String),

    // de-referencing a variable e.g. *x;
    DeRef(String),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Return(Expression),
    If(Expression, Vec<Stmt>, Option<Vec<Stmt>>),
    Assign(Assignment),
    ReAssign(Local),
    Call(Call),
}
