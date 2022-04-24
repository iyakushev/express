use std::{fmt, iter::Peekable};
use core::str::Chars;

/// An error reported by the parser.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken(char),
    MissingRParen(i32),
    MissingArgument,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::UnexpectedToken(i) => write!(f, "Unexpected token at byte {}.", i),
            ParseError::MissingRParen(i) => write!(
                f,
                "Missing {} right parenthes{}.",
                i,
                if i == 1 { "is" } else { "es" }
            ),
            ParseError::MissingArgument => write!(f, "Missing argument at the end of expression."),
        }
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::UnexpectedToken(_) => "unexpected token",
            ParseError::MissingRParen(_) => "missing right parenthesis",
            ParseError::MissingArgument => "missing argument",
        }
    }
}




/// Mathematical operations.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Operation {
    Plus,
    Minus,
    Times,
    Div,
    Rem,
    Pow,
    Fact,
}

/// Expression tokens.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Binary(Operation),
    Unary(Operation),

    LParen,
    RParen,
    Comma,

    Number(f64),
    Ident(String),
    Error(ParseError),
}


struct Lexer<'l>{
    tokstream: Peekable<Chars<'l>>
}

impl<'l> Lexer<'l>{

    /// Reads input string and produces token iterator
    pub fn tokenize(input: &str) -> impl Iterator + '_ {
        let mut lex = Lexer {
            tokstream: input.chars().into_iter().peekable()
        };
        lex.tokstream.map(|current_char| {
            match current_char {
                '+' => Token::Binary(Operation::Plus),
                '-' => Token::Binary(Operation::Minus),
                '*' => Token::Binary(Operation::Pow),
                '/' => Token::Binary(Operation::Div),
                '%' => Token::Binary(Operation::Rem),
                '!' => Token::Binary(Operation::Fact),
                'A'..='Z' | 'a'..='z' => unimplemented!(),
                '1'..='9' => unimplemented!(),
                '(' => Token::LParen,
                ')' => Token::RParen,
                ',' => Token::Comma,
                _ => Token::Error(ParseError::UnexpectedToken(current_char))
            }
        }).peekable()
    }
}


#[cfg(test)]
mod tests {

    pub fn test_simple_expr() {
        let expr = "2 + 2 * 2";
    }

    pub fn test_paren() {
        let expr = "(2 + 2) * 2";
    }

    pub fn test_ident() {
        let expr = "name() + 2";
    }

    pub fn test_func() {
        let expr = "fn(arg, arg2) ** 2";
    }
}