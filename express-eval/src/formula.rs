use crate::{ctx::Context, ir::IRNode};
use express::lang::{ast::Visit, parser::parse_expression};
use express::types::Type;

pub struct Formula {
    _name: String,
    pub ast: IRNode,
    pub result: Option<Type>,
}

impl Iterator for Formula {
    type Item = Type;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl Formula {
    pub fn new(name: &str, expression: &str, eval_ctx: &Context) -> Result<Self, String> {
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
            _name: name.to_string(),
            ast: eval_ctx.visit_expr(ast)?,
            result: None,
        })
    }
}
