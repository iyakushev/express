#[allow(dead_code)]
use nom::{
    bytes::complete::tag,
    character::{complete::one_of, is_alphanumeric},
    combinator::opt,
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
    if let (inp, Some(_)) = opt(tag("**"))(input)? {
        return Ok((inp, Operation::Power));
    }
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

fn parse_literal(input: &str) -> IResult<&str, Literal> {
    unimplemented!()
}

fn parse_unary(input: &str) -> IResult<&str, Expression> {
    let (input, op) = parse_op(input)?;
    unimplemented!()
}

fn parse_binary(input: &str) -> IResult<&str, Expression> {
    unimplemented!()
}

/// Parses function expressions like `foo(<Expression, *>).*`
fn parse_function(input: &str) -> IResult<&str, Expression> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_op {
        ($str:expr, $tok:expr) => {
            let (_, t) = parse_op($str).unwrap();
            assert_eq!(t, $tok);
        };
    }

    #[test]
    fn test_op() {
        test_op!("+", Operation::Plus);
        test_op!("-", Operation::Minus);
        test_op!("*", Operation::Times);
        test_op!("/", Operation::Divide);
        test_op!("!", Operation::Factorial);
        test_op!("**", Operation::Power);
        test_op!("**garbage", Operation::Power);
    }
}
