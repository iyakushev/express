use std::str::FromStr;

use crate::lex::{Lexer, Token};

struct Parser<'p> {
    source: &'p str,
    current_tok: Token,
    token_it: Lexer<'p>,
}

pub trait Express {
    fn parse(&self) -> AST;
}
