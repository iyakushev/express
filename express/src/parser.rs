use nom::{
    branch::alt,
    character::complete::{alpha0, alpha1, digit1},
    combinator::{map, map_res},
    sequence::{preceded, terminated},
};
#[allow(dead_code)]
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
    Function { name: String, args: Vec<Expression> },
    BinOp(Box<Expression>, Box<Expression>, Operation),
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

fn parse_number(input: &str) -> IResult<&str, Literal> {
    alt((
        map_res(digit1, |digit_str: &str| {
            digit_str.parse::<f64>().map(Literal::Number)
        }),
        map(preceded(tag("-"), digit1), |digit_str: &str| {
            Literal::Number(-1.0 * digit_str.parse::<f64>().unwrap())
        }),
    ))(input)
}

/// Parses any given identifier which is alphabetic
/// ```no_run
/// assert_eq!(parse_ident("abc"), Ok(("", Literal::Ident(String("abc")))))
/// assert_eq!(parse_ident("1abc"), Err(...))
/// ```
fn parse_ident(input: &str) -> IResult<&str, Literal> {
    map(terminated(tag("("), alpha1), |ident: &str| {
        Literal::Ident(ident.to_string())
    })(input)
}

fn parse_literal(input: &str) -> IResult<&str, Literal> {
    alt((parse_number, parse_ident))(input)
}

fn parse_binary(input: &str) -> IResult<&str, Expression> {
    unimplemented!()
}

fn parse_iterable<'i>(
    input: &'i str,
    start_encl: &str,
    end_encl: &str,
) -> IResult<&'i str, Vec<Expression>> {
    unimplemented!()
}

/// Parses function expressions like `foo(<Expression, *>).*`
fn parse_function(input: &str) -> IResult<&str, Expression> {
    let (input, id) = parse_literal(input)?;
    match id {
        Literal::Number(d) => Err(nom::Err::Error(nom::error::Error::new(
            &format!(
                "Function names can not start with a digit: {}",
                d.to_string()
            ),
            nom::error::ErrorKind::AlphaNumeric,
        ))),
        Literal::Ident(name) => {
            let (input, _) = tag("(")(input)?;
            let (input, args) = parse_iterable(input, "(", ")")?;
            let (input, _) = tag(")")(input)?;
            Ok((input, Expression::Function { name, args }))
        }
    }
}

/// Parses function expressions like `foo(<Expression, *>).*`
fn parse_expression(input: &str) -> IResult<&str, Expression> {
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
