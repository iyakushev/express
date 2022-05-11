#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Ident(String),
    Number(f64),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    Plus,
    Minus,
    Times,
    Divide,
    Power,
    Factorial,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Const(Literal),
    Function {
        name: Literal,
        args: Vec<Expression>,
    },
    BinOp(Box<Expression>, Box<Expression>, Operation),
    UnOp(Operation, Box<Expression>),
}
