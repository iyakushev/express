use express::lang::{
    ast::{Expression, Literal},
    parser::parse_expression,
};
use express::types::{Function, Type, FN_REGISTRY};

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
}
