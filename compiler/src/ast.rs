#[derive(Debug)]
pub struct Assignment {
    pub target: String,
    pub value: String,
}

#[derive(Debug)]
pub struct Call {
    pub func: String,
    pub args: Vec<String>,
}

impl Call {
    pub fn new(func: String, args: Vec<String>) -> Call {
        Call { func, args }
    }
}

impl Assignment {
    pub fn new(target: String, value: String) -> Assignment {
        Assignment { target, value }
    }
}

#[derive(Debug)]
pub enum ParseResult {
    Assign(Assignment),
    Call(Call),
}
