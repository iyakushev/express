use std::sync::Arc;

use crate::{ctx::Context, ir::IRNode};
use express::lang::{ast::Visit, parser::parse_expression};
use express::types::Type;

type Link = Option<Arc<Formula>>;

#[derive(PartialEq, Debug)]
pub struct Formula {
    pub ast: IRNode,
    pub next: Link,
    pub parents: Vec<Arc<Formula>>,
    pub result: Option<Type>,
}

impl Iterator for Formula {
    type Item = Type;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl Formula {
    pub fn new(expression: &str, eval_ctx: &Context) -> Result<Self, String> {
        let (_, ast) = match parse_expression(expression) {
            Ok(it) => it,
            Err(err) => return Err(format!("Failed to parse expression. Reason: {}", err)),
        };
        Ok(Self {
            next: None,
            ast: eval_ctx.visit_expr(ast)?,
            parents: vec![],
            result: None,
        })
    }
}
