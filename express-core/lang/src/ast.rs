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

impl Operation {
    #[inline]
    pub fn eval(&self, lhs: f64, rhs: f64) -> f64 {
        match self {
            Operation::Plus => lhs + rhs,
            Operation::Minus => lhs - rhs,
            Operation::Times => lhs * rhs,
            Operation::Divide => lhs / rhs,
            Operation::Power => rhs.powf(lhs),
            Operation::Factorial => (rhs as usize..1).fold(1.0, |acc, val| acc * val as f64),
        }
    }

    #[inline]
    pub fn unary_eval(&self, rhs: f64) -> f64 {
        match self {
            Operation::Factorial => (rhs as usize..1).fold(1.0, |acc, val| acc * val as f64),
            Operation::Minus => -1.0 * rhs,
            _ => rhs,
        }
    }
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

/// Provides a Visitor pattern interface to the Expression
pub trait Visit<T> {
    type Returns;

    fn visit_const(&self, cnst: T) -> Result<Self::Returns, String>;
    fn visit_fn(&self, xfn: T) -> Result<Self::Returns, String>;
    fn visit_binop(&self, bin: T) -> Result<Self::Returns, String>;
    fn visit_unop(&self, un: T) -> Result<Self::Returns, String>;
    fn visit_expr(&self, expr: T) -> Result<Self::Returns, String>;
}
