pub enum Keyword {
    Pass,
    Return,
}

#[derive(Debug)]
pub struct Assignment {
    pub target: Name,
    pub value: Atom,
}

/// Function declaration AST node
#[derive(Debug)]
pub struct FnDecl {
    pub name: String,
    pub args: Vec<Name>,
    pub body: Vec<Expression>,
}

#[derive(Debug)]
pub struct Call {
    pub func: String,
    pub args: Vec<Atom>,
}

#[derive(Debug)]
pub struct Name {
    pub ident: String,
}

#[derive(Debug)]
pub enum ComparisonOperator {
    Eq,
    NotEq,
    Gt,
    Lt,
}

#[derive(Debug)]
pub enum Atom {
    Name(Name),
    Constant(Constant),
}

#[derive(Debug)]
pub struct Compare {
    pub left: Atom,
    pub op: ComparisonOperator,
    pub right: Atom,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Expression {
    Assign(Assignment),
    Call(Call),
    If(Compare, Vec<Expression>, Option<Vec<Expression>>),
}
