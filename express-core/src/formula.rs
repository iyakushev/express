use lang::{
    ast::{Expression, Literal},
    parser::parse_expression,
};
use types::{Function, Type, FN_REGISTRY};
pub use xmacro;

pub struct Formula {
    name: String,
    ast: Expression,
    result: Option<Type>,
}

impl Iterator for Formula {
    type Item = Type;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl Formula {
    pub fn new(name: &str, expression: &str) -> Result<Self, String> {
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
            ast,
            result: None,
        })
    }

    fn lookup_fn<'a>(&self, e_fn: &'a Expression) -> Result<&'a Function, String> {
        if let Expression::Function { name, args } = e_fn {
            match name {
                Literal::Ident(id) => {
                    if let Some(f) = FN_REGISTRY.get(id.as_str()) {
                        Ok(f)
                    } else {
                        Err(format!(
                            "Failed to find function with a matching name: {}",
                            id
                        ))
                    }
                }
                _ => unreachable!(),
            }
        } else {
            Err("Given expression is not a function".to_string())
        }
    }

    fn optimize(&mut self) {}
}
