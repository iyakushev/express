use crate::{ctx::Context, ir::IRNode};
use express::lang::{ast::Visit, parser::parse_expression};
use express::types::Type;
use std::rc::Rc;

pub struct Formula {
    name: String,
    ast: IRNode,
    result: Option<Type>,
    eval_ctx: Rc<Context>,
}

impl Iterator for Formula {
    type Item = Type;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl Formula {
    pub fn new(name: &str, expression: &str, eval_ctx: Rc<Context>) -> Result<Self, String> {
        let (_, ast) = match parse_expression(expression) {
            Ok(it) => it,
            Err(err) => {
                return Err(format!(
                    "Failed to parse expression '{}'. Reason: {}",
                    name, err
                ))
            }
        };
        Ok(Self {
            name: name.to_string(),
            ast: eval_ctx.visit_expr(ast)?,
            result: None,
            eval_ctx,
        })
    }
}
