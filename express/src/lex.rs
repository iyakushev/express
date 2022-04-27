use core::str::Chars;
use std::{fmt, iter::Peekable};

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

struct Lexer<'l> {
    source: Peekable<Chars<'l>>,
    lineno: usize,
    column: usize,
}

impl<'l> Lexer<'l> {
    pub fn new(source: &'l str) -> Self {
        Self {
            source: source.chars().peekable(),
            lineno: 1,
            column: 1,
        }
    }
}

impl<'l> Lexer<'l> {
    fn consume_num(&mut self, first_char: char) {}
}

impl<'l> Iterator for Lexer<'l> {
    type Item = Token;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut current_char = self.source.next()?;
        self.column += 1;
        while current_char.is_whitespace() {
            if current_char == '\n' {
                self.column = 1;
                self.lineno += 1;
            }
            current_char = self.source.next()?;
            self.column += 1;
        }
        let tok = match current_char {
            '+' => Token::Binary(Operation::Plus),
            '-' => Token::Binary(Operation::Minus),
            '*' => {
                if let Some('*') = self.source.peek() {
                    self.column += 1;
                    return Some(Token::Binary(Operation::Pow));
                }
                Token::Binary(Operation::Times)
            }
            '/' => Token::Binary(Operation::Div),
            '%' => Token::Binary(Operation::Rem),
            '!' => Token::Binary(Operation::Fact),
            'A'..='Z' | 'a'..='z' => unimplemented!(),
            '1'..='9' => unimplemented!(),
            '(' => Token::LParen,
            ')' => Token::RParen,
            ',' => Token::Comma,
            _ => Token::Error(ParseError::UnexpectedToken(current_char)),
        };
        Some(tok)
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
