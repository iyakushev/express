#![allow(dead_code)]
use nom::{
    branch::alt,
    character::complete::{alpha1, char, multispace0},
    combinator::{cut, map},
    error::context,
    multi::separated_list0,
    number::complete::double,
    sequence::{delimited, preceded},
};
use nom::{bytes::complete::tag, character::complete::one_of, combinator::opt, IResult};

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
    // UnOp(Box<Expression>, Box<Expression>),
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

/// Parses number as a floating point. Any fp notation is valid
fn parse_number(input: &str) -> IResult<&str, Literal> {
    alt((
        map(double, |num: f64| Literal::Number(num)),
        map(preceded(tag("-"), double), |num: f64| {
            Literal::Number(-1.0 * num)
        }),
    ))(input)
}

/// Parses any given identifier which is alphabetic
/// ```ignore
/// assert_eq!(parse_ident("abc"), Ok(("", Literal::Ident(String("abc")))))
/// assert_eq!(parse_ident("1abc"), Err(...))
/// ```
fn parse_ident(input: &str) -> IResult<&str, Literal> {
    map(alpha1, |ident: &str| Literal::Ident(ident.to_string()))(input)
}

fn parse_literal(input: &str) -> IResult<&str, Literal> {
    alt((parse_number, parse_ident))(input)
}

fn parse_const(input: &str) -> IResult<&str, Expression> {
    map(parse_literal, |lit| Expression::Const(lit))(input)
}

/// Parses any binary arithmetic expression like
/// ```ignore
/// assert_eq!(parse_binary("1 + 1"), Expression::BinOp(
///                                      Box::new(Expression::Const(
///                                          Literal::Number(1.0))),
///                                      Box::new(Expression::Const(
///                                          Literal::Number(1.0))),
///                                      Operation::Plus)
/// ```
fn parse_binary(input: &str) -> IResult<&str, Expression> {
    let (input, lexpr) = parse_expression(input)?;
    let (input, op) = preceded(multispace0, parse_op)(input)?;
    let (input, rexpr) = parse_expression(input)?;
    Ok((
        input,
        Expression::BinOp(Box::new(lexpr), Box::new(rexpr), op),
    ))
}

/// Parses function expressions like `foo(<Expression, *>).*`
fn parse_function<'a>(input: &str) -> IResult<&str, Expression> {
    let (input, fn_name) = parse_ident(input)?;
    map(
        delimited(
            char('('),
            preceded(multispace0, separated_list0(char(','), parse_expression)),
            context("closing paren", cut(preceded(multispace0, char(')')))),
        ),
        move |result: Vec<Expression>| Expression::Function {
            name: fn_name.clone(),
            args: result,
        },
    )(input)
}

/// Parses function expressions like
/// EXPRESSION := FUNCTION | CONST | BINARY | EXPRESSION
pub fn parse_expression(input: &str) -> IResult<&str, Expression> {
    preceded(
        multispace0,
        alt((parse_function, parse_const, parse_binary)),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_op {
        ($parser:expr, $str:expr => $tok:expr) => {
            let (_, t) = $parser($str).unwrap();
            assert_eq!(t, $tok);
        };
        ($parser:expr, $str:expr => $tok:expr, $err:expr) => {
            match $parser($str) {
                Ok((_, t)) => assert_eq!(t, $tok),
                Err(e) => assert_eq!(e, $err),
            }
        };
    }

    #[test]
    fn test_op() {
        test_op!(parse_op, "+" => Operation::Plus);
        test_op!(parse_op, "-" => Operation::Minus);
        test_op!(parse_op, "*" => Operation::Times);
        test_op!(parse_op, "/" => Operation::Divide);
        test_op!(parse_op, "!" => Operation::Factorial);
        test_op!(parse_op, "**" => Operation::Power);
        test_op!(parse_op, "**garbage" => Operation::Power);
    }

    #[test]
    fn test_id() {
        test_op!(parse_ident, "hello" => Literal::Ident("hello".to_string()));
        test_op!(parse_ident, "hello world" => Literal::Ident("hello".to_string()));
    }

    #[test]
    fn test_num() {
        test_op!(parse_number, "12" => Literal::Number(12.0f64));
        test_op!(parse_number, "22.22" => Literal::Number(22.22f64));
        test_op!(parse_number, "1e-10" => Literal::Number(1e-10f64));
    }

    #[test]
    fn test_bin() {
        test_op!(parse_binary, "12 + 12" =>  Expression::BinOp(
                                              Box::new(Expression::Const(
                                                  Literal::Number(12.0))),
                                              Box::new(Expression::Const(
                                                  Literal::Number(12.0))),
                                              Operation::Plus)
        );
        test_op!(parse_binary, "foo() + 12" =>  Expression::BinOp(
                                            Box::new(Expression::Function
                                                     { name: Literal::Ident("foo".to_string()),
                                                       args: vec![] }),
                                              Box::new(Expression::Const(
                                                  Literal::Number(12.0))),
                                              Operation::Plus)
        );
    }

    //#[test]
    //fn test_bin_expr() {
    //    test_op!(parse_expression, "12 + 12 * 2" =>  Expression::BinOp(
    //                                            Box::new(Expression::BinOp(
    //                                                Box::new(Expression::Const(
    //                                                    Literal::Number(12.0))),
    //                                                Box::new(Expression::Const(
    //                                                    Literal::Number(12.0))),
    //                                                Operation::Plus)),
    //                                            Box::new(Expression::Const(Literal::Number(2.0))),
    //                                            Operation::Times)
    //    );
    //}

    #[test]
    fn test_fn() {
        test_op!(parse_function, "foo()" => Expression::Function { name: Literal::Ident("foo".to_string()), args: vec![] });
        test_op!(parse_function, "foo(bar(), baz())" => Expression::Function {
        name: Literal::Ident("foo".to_string()),
        args: vec![
            Expression::Function { name: Literal::Ident("bar".to_string()), args: vec![] },
            Expression::Function { name: Literal::Ident("baz".to_string()), args: vec![] },
        ]});
    }
}
