#![allow(dead_code)]
use crate::ast::*;
use nom::branch::permutation;
use nom::bytes::complete::{tag, take_while1};
use nom::character::is_alphanumeric;
use nom::{
    branch::alt,
    character::complete::{alpha1, char, multispace0},
    combinator::{cut, map},
    error::context,
    multi::{fold_many0, separated_list0},
    number::complete::double,
    sequence::{delimited, pair, preceded},
    IResult,
};

/// Parses number as a floating point. Any fp notation is valid
fn parse_number(input: &str) -> IResult<&str, Literal> {
    map(double, |num: f64| Literal::Number(num))(input)
}

fn _is_valid_ident(chr: char) -> bool {
    is_alphanumeric(chr as u8) || chr == '_'
}

/// Parses any given identifier which is alphabetic
/// ```ignore
/// assert_eq!(parse_ident("abc"), Ok(("", Literal::Ident(String("abc")))))
/// assert_eq!(parse_ident("1abc"), Err(...))
/// ```
fn parse_ident(input: &str) -> IResult<&str, Literal> {
    map(take_while1(_is_valid_ident), |ident: &str| {
        Literal::Ident(ident.to_string())
    })(input)
}

fn parse_literal(input: &str) -> IResult<&str, Literal> {
    alt((parse_number, parse_ident))(input)
}

/// Operand can be a literal: __12__, __12.23__, __PI__
/// or it can be a function: __ema(...)__
fn parse_operand(input: &str) -> IResult<&str, Expression> {
    let (input, lit) = parse_literal(input)?;
    if matches!(lit, Literal::Ident(_)) && input.chars().peekable().peek() == Some(&'(') {
        return parse_function(input, lit);
    }
    Ok((input, Expression::Const(lit)))
}

fn parse_parens(input: &str) -> IResult<&str, Expression> {
    delimited(
        multispace0,
        delimited(char('('), parse_expression, char(')')),
        multispace0,
    )(input)
}

/// Parses either a const/fn operand or expression inside parens
fn parse_factor(input: &str) -> IResult<&str, Expression> {
    preceded(multispace0, alt((parse_operand, parse_parens, parse_unary)))(input)
}

/// Parses binary expression with exponents: __2**2__
fn parse_bin_exp(input: &str) -> IResult<&str, Expression> {
    let (input, lhs) = parse_factor(input)?;
    fold_many0(
        preceded(multispace0, preceded(tag("**"), parse_factor)),
        move || lhs.clone(),
        |mut acc: Expression, rhs| {
            acc = Expression::BinOp(Box::new(rhs), Box::new(acc), Operation::Power);
            acc
        },
    )(input)
    // Ok((input, lhs))
}

fn parse_bin_term(input: &str) -> IResult<&str, Expression> {
    let (input, lhs) = parse_bin_exp(input)?;
    fold_many0(
        preceded(
            multispace0,
            pair(alt((char('*'), char('/'))), parse_bin_exp),
        ),
        move || lhs.clone(),
        |mut acc: Expression, (op, rhs)| {
            let op = if matches!(op, '*') {
                Operation::Times
            } else {
                Operation::Divide
            };
            acc = Expression::BinOp(Box::new(acc), Box::new(rhs), op);
            acc
        },
    )(input)
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
    let (input, lhs) = parse_bin_term(input)?;
    fold_many0(
        preceded(
            multispace0,
            pair(alt((char('+'), char('-'))), parse_bin_term),
        ),
        move || lhs.clone(),
        |mut acc: Expression, (op, rhs)| {
            let op = if matches!(op, '+') {
                Operation::Plus
            } else {
                Operation::Minus
            };
            acc = Expression::BinOp(Box::new(acc), Box::new(rhs), op);
            acc
        },
    )(input)
}

/// Returns unary expression representation like: __-12__, __-ema(...)__
fn parse_unary(input: &str) -> IResult<&str, Expression> {
    map(
        pair(alt((char('-'), char('!'))), parse_operand),
        |(op, rhs)| {
            Expression::UnOp(
                if op == '-' {
                    Operation::Minus
                } else {
                    Operation::Factorial
                },
                Box::new(rhs),
            )
        },
    )(input)
}

fn _parse(input: &str) -> IResult<&str, Expression> {
    parse_binary(input)
}

/// Parses function expressions like `foo(<Expression, *>).*`
fn parse_function<'a>(input: &str, fn_name: Literal) -> IResult<&str, Expression> {
    // let (input, fn_name) = parse_ident(input)?;
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
/// EXPRESSION := FUNCTION | CONST | BINARY
pub fn parse_expression(input: &str) -> IResult<&str, Expression> {
    preceded(multispace0, _parse)(input)
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
    fn test_fn() {
        test_op!(parse_operand, "foo()" => Expression::Function { name: Literal::Ident("foo".to_string()), args: vec![] });
        test_op!(parse_operand, "foo(bar(), baz())" => Expression::Function {
        name: Literal::Ident("foo".to_string()),
        args: vec![
            Expression::Function { name: Literal::Ident("bar".to_string()), args: vec![] },
            Expression::Function { name: Literal::Ident("baz".to_string()), args: vec![] },
        ]});

        test_op!(parse_operand, "foo(22, 2 + 2)" => Expression::Function {
        name: Literal::Ident("foo".to_string()),
        args: vec![
            Expression::Const(Literal::Number(22.0)) ,
            Expression::BinOp(Box::new(Expression::Const(Literal::Number(2.0))),
                              Box::new(Expression::Const(Literal::Number(2.0))),
                              Operation::Plus),
        ]});
    }

    #[test]
    fn test_unary() {
        test_op!(parse_expression, "-12" =>
            Expression::Const(Literal::Number(-12.0))
        );
        test_op!(parse_expression, "-ema()" => Expression::UnOp(
            Operation::Minus,
            Box::new(
                Expression::Function { name: Literal::Ident("ema".to_string()), args: vec![] }))
        );
        test_op!(parse_expression, "!12" =>
                Expression::UnOp(
                    Operation::Factorial,
                    Box::new(
                        Expression::Const(
                            Literal::Number(12.0)))
        ));
    }

    #[test]
    fn test_bin_exp() {
        test_op!(parse_bin_exp, "2 ** 3" => Expression::BinOp(
            Box::new(Expression::Const(Literal::Number(3.0))),
            Box::new(Expression::Const(Literal::Number(2.0))),
            Operation::Power));

        test_op!(parse_bin_exp, "2 ** 3 ** 4" => Expression::BinOp(
            Box::new(Expression::Const(Literal::Number(4.0))),
            Box::new(Expression::BinOp(
                Box::new(Expression::Const(Literal::Number(3.0))),
                Box::new(Expression::Const(Literal::Number(2.0))),
                Operation::Power)),
            Operation::Power));
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

        test_op!(parse_binary, "12 + foo()" =>  Expression::BinOp(
                                            Box::new(Expression::Const(
                                                  Literal::Number(12.0))),
                                            Box::new(Expression::Function
                                                     { name: Literal::Ident("foo".to_string()),
                                                       args: vec![] }),
                                              Operation::Plus)
        );
    }

    #[test]
    fn test_bin_expr() {
        test_op!(parse_expression, "2 + 2 * 2" =>  Expression::BinOp(
                                                Box::new(Expression::Const(Literal::Number(2.0))),
                                                Box::new(Expression::BinOp(
                                                    Box::new(Expression::Const(Literal::Number(2.0))),
                                                    Box::new(Expression::Const(Literal::Number(2.0))),
                                                    Operation::Times)
                                                ),
                                                Operation::Plus)
        );
        test_op!(parse_expression, "(2 + 2) * 2" =>  Expression::BinOp(
                                                Box::new(Expression::BinOp(
                                                    Box::new(Expression::Const(Literal::Number(2.0))),
                                                    Box::new(Expression::Const(Literal::Number(2.0))),
                                                    Operation::Plus)
                                                ),
                                                Box::new(Expression::Const(Literal::Number(2.0))),
                                                Operation::Times)
        );

        test_op!(parse_expression, "3.14 + (3 - 2)" =>  Expression::BinOp(
                                                Box::new(Expression::Const(Literal::Number(3.14))),
                                                Box::new(Expression::BinOp(
                                                    Box::new(Expression::Const(Literal::Number(3.0))),
                                                    Box::new(Expression::Const(Literal::Number(2.0))),
                                                    Operation::Minus)
                                                ),
                                                Operation::Plus)
        );
    }

    #[test]
    fn test_const() {
        test_op!(parse_expression, "12" => Expression::Const(Literal::Number(12.0)));
    }
}
