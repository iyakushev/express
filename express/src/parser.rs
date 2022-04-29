use nom::{
    bytes::complete::{tag, take_while_m_n},
    character::complete::one_of,
    sequence::tuple,
    IResult,
};

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
    Function(Vec<Expression>),
    BinOp(Box<Expression>, Box<Expression>, Operation),
    UnOp(Box<Expression>, Operation),
}

/// Parses operation token: `+, -, *, /, **, !`
fn parse_op(input: &str) -> IResult<&str, Operation> {
    let (inp, tok) = one_of("+-*/!")(input)?;
    let tok = match tok {
        '+' => Operation::Plus,
        '-' => Operation::Minus,
        '/' => Operation::Divide,
        '*' => Operation::Times,
        '!' => Operation::Factorial,
        _ => unreachable!(),
    };
    Ok((inp, tok))
}
