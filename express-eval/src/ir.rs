use express::{lang::ast::Operation, types::Callable};
use std::rc::Rc;

pub enum IRNode {
    Number(f64),
    Function(Rc<dyn Callable>, Vec<IRNode>),
    BinOp(Box<IRNode>, Box<IRNode>, Operation),
    UnOp(Box<IRNode>, Operation),
}

impl From<IRNode> for f64 {
    fn from(val: IRNode) -> Self {
        match val {
            IRNode::Number(f) => f,
            _ => panic!("Failed to convert IRNode to f64"),
        }
    }
}
