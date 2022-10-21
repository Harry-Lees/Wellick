#[derive(Debug)]
pub enum ExprKind {
    Add,
    Sub,
    Mult,
    Div,
}

#[derive(Debug)]
pub struct Expr {
    value: ExprKind,
    left: Option<Box<Expr>>,
    right: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct Constant {
    value: i32,
}

impl Constant {
    pub fn new(value: i32) -> Constant {
        Constant { value }
    }
}

#[derive(Debug)]
pub struct BinOp {
    value: ExprKind,
    left: Constant,
    right: Constant,
}

impl BinOp {
    pub fn new(value: ExprKind, left: Constant, right: Constant) -> BinOp {
        BinOp { value, left, right }
    }
}