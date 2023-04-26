use std::{
    fmt::{self, Formatter},
    str::FromStr,
};

#[derive(Clone)]
pub struct IntegerLiteral {
    token: String,
}

#[derive(Clone)]
pub struct FloatLiteral {
    token: String,
}

impl fmt::Debug for IntegerLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("IntegerLiteral {}", self.token))
    }
}

impl fmt::Debug for FloatLiteral {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("FloatLiteral {}", self.token))
    }
}

impl IntegerLiteral {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }

    pub fn base10_parse<T: FromStr>(&self) -> Result<T, T::Err> {
        self.token.parse::<T>()
    }
}

impl FloatLiteral {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }

    pub fn base10_parse<T: FromStr>(&self) -> Result<T, T::Err> {
        self.token.parse::<T>()
    }
}

/// Types supported by the language which also hold the corresponding
/// value, used in AST constructs like assignments
#[derive(Debug, Clone)]
pub enum Literal {
    Float(FloatLiteral),
    Integer(IntegerLiteral),
}

#[derive(Debug, Clone)]
pub struct Pointer {
    pub ty: EmptyType,
    pub mutable: bool,
}

impl Pointer {
    pub fn new(ty: EmptyType, mutable: bool) -> Self {
        Pointer { ty, mutable }
    }
}

#[derive(Debug, Clone)]
pub enum IntegerType {
    I32,
    I64,
    PointerSize,
}

#[derive(Debug, Clone)]
pub enum FloatType {
    F32,
    F64,
}

#[derive(Debug, Clone)]
pub enum EmptyType {
    Float(FloatType),
    Integer(IntegerType),
    Pointer(Box<Pointer>),
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub target: Name,
    pub value: Expression,
    pub var_type: EmptyType,
    pub mutable: bool,
}

impl Assignment {
    pub fn new(target: Name, var_type: EmptyType, value: Expression, mutable: bool) -> Self {
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

#[derive(Debug, Clone)]
pub struct AddressOf {
    pub name: String,
    pub mutable: bool,
}

impl AddressOf {
    pub fn new(name: String, mutable: bool) -> Self {
        AddressOf { name, mutable }
    }
}

/// Function declaration AST node
#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub args: Vec<FnArg>,
    pub ret_type: EmptyType,
    pub body: Vec<Stmt>,
}

impl FnDecl {
    pub fn new(name: String, args: Vec<FnArg>, ret_type: EmptyType, body: Vec<Stmt>) -> Self {
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
    Literal(Literal),

    // A variable identifier e.g. x;
    Identifier(String),

    // Address-of a variable e.g. &x; gets the address of x.
    AddressOf(AddressOf),

    // de-referencing a variable e.g. *x;
    DeRef(String),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Return(Expression),
    If(Expression, Vec<Stmt>),
    Assign(Assignment),
    ReAssign(Local),
    Call(Call),
}
