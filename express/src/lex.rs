use core::str::Chars;
use std::{fmt, iter::Peekable, num::ParseFloatError};

/// An error reported by the parser.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken(char),
    FailedToParseFloat(String),
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
            _ => write!(f, " "),
        }
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::UnexpectedToken(_) => "unexpected token",
            ParseError::MissingRParen(_) => "missing right parenthesis",
            ParseError::MissingArgument => "missing argument",
            _ => " ",
        }
    }
}

/// Mathematical operations.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Arithm {
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
    Op(Arithm),

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
    fn consume_num(&mut self, first_char: char) -> Token {
        let mut result = String::with_capacity(20);
        result.push(first_char);
        while let Some(ch) = self.source.peek() {
            if !ch.is_numeric() && *ch != '.' {
                break;
            }
            result.push(self.source.next().unwrap());
        }
        if let Ok(r) = result.parse::<f64>() {
            Token::Number(r)
        } else {
            Token::Error(ParseError::FailedToParseFloat(result))
        }
    }

    fn consume_ident(&mut self, first_char: char) -> Token {
        let mut result = String::with_capacity(20);
        result.push(first_char);
        while let Some(ch) = self.source.peek() {
            if !ch.is_alphanumeric() && *ch != '_' {
                break;
            }
            result.push(self.source.next().unwrap());
        }
        Token::Ident(result)
    }
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
            '+' => Token::Op(Arithm::Plus),
            '-' => Token::Op(Arithm::Minus),
            '*' => {
                if let Some('*') = self.source.peek() {
                    self.column += 1;
                    self.source.next();
                    return Some(Token::Op(Arithm::Pow));
                }
                Token::Op(Arithm::Times)
            }
            '/' => Token::Op(Arithm::Div),
            '%' => Token::Op(Arithm::Rem),
            '!' => Token::Op(Arithm::Fact),
            'A'..='Z' | 'a'..='z' => self.consume_ident(current_char),
            '1'..='9' => self.consume_num(current_char),
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
    use super::*;

    macro_rules! tokenize {
        ($exp:expr) => {
            Lexer::new($exp).into_iter().collect::<Vec<Token>>()
        };
    }

    #[test]
    pub fn test_simple_expr() {
        let tokens = tokenize!("2 + 2 * 2");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::Number(2.0));
        assert_eq!(tokens[1], Token::Op(Arithm::Plus));
        assert_eq!(tokens[2], Token::Number(2.0));
        assert_eq!(tokens[3], Token::Op(Arithm::Times));
        assert_eq!(tokens[4], Token::Number(2.0));
    }

    #[test]
    pub fn test_paren() {
        let tokens = tokenize!("(2 + 2) * 2");
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0], Token::LParen);
        assert_eq!(tokens[1], Token::Number(2.0));
        assert_eq!(tokens[2], Token::Op(Arithm::Plus));
        assert_eq!(tokens[3], Token::Number(2.0));
        assert_eq!(tokens[4], Token::RParen);
        assert_eq!(tokens[5], Token::Op(Arithm::Times));
        assert_eq!(tokens[6], Token::Number(2.0));
    }

    #[test]
    pub fn test_ident() {
        let tokens = tokenize!("name() + 2");

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::Ident(String::from("name")));
        assert_eq!(tokens[1], Token::LParen);
        assert_eq!(tokens[2], Token::RParen);
        assert_eq!(tokens[3], Token::Op(Arithm::Plus));
        assert_eq!(tokens[4], Token::Number(2.0));
    }

    #[test]
    pub fn test_func() {
        let tokens = tokenize!("fn(arg, arg2) ** 2");

        assert_eq!(tokens.len(), 8);
        assert_eq!(tokens[0], Token::Ident(String::from("fn")));
        assert_eq!(tokens[1], Token::LParen);
        assert_eq!(tokens[2], Token::Ident(String::from("arg")));
        assert_eq!(tokens[3], Token::Comma);
        assert_eq!(tokens[4], Token::Ident(String::from("arg2")));
        assert_eq!(tokens[5], Token::RParen);
        assert_eq!(tokens[6], Token::Op(Arithm::Pow));
        assert_eq!(tokens[7], Token::Number(2.0));
    }
}
