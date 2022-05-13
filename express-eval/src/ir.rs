use express::{lang::ast::Operation, types::Callable};
use std::{fmt::Debug, rc::Rc};

pub enum IRNode {
    Number(f64),
    Function(Rc<dyn Callable>, Vec<IRNode>),
    BinOp(Box<IRNode>, Box<IRNode>, Operation),
    UnOp(Box<IRNode>, Operation),
}

impl PartialEq for IRNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Function(_, l1), Self::Function(_, r1)) => l1 == r1,
            (Self::BinOp(l0, l1, l2), Self::BinOp(r0, r1, r2)) => l0 == r0 && l1 == r1 && l2 == r2,
            (Self::UnOp(l0, l1), Self::UnOp(r0, r1)) => l0 == r0 && l1 == r1,
            (IRNode::Number(_), IRNode::Function(_, _)) => todo!(),
            (IRNode::Number(_), IRNode::BinOp(_, _, _)) => todo!(),
            (IRNode::Number(_), IRNode::UnOp(_, _)) => todo!(),
            (IRNode::Function(_, _), IRNode::Number(_)) => todo!(),
            (IRNode::Function(_, _), IRNode::BinOp(_, _, _)) => todo!(),
            (IRNode::Function(_, _), IRNode::UnOp(_, _)) => todo!(),
            (IRNode::BinOp(_, _, _), IRNode::Number(_)) => todo!(),
            (IRNode::BinOp(_, _, _), IRNode::Function(_, _)) => todo!(),
            (IRNode::BinOp(_, _, _), IRNode::UnOp(_, _)) => todo!(),
            (IRNode::UnOp(_, _), IRNode::Number(_)) => todo!(),
            (IRNode::UnOp(_, _), IRNode::Function(_, _)) => todo!(),
            (IRNode::UnOp(_, _), IRNode::BinOp(_, _, _)) => todo!(),
        }
    }
}

impl Debug for IRNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(arg0) => f.debug_tuple("Number").field(arg0).finish(),
            Self::Function(_, arg1) => f.debug_tuple("Function").field(arg1).finish(),
            Self::BinOp(arg0, arg1, arg2) => f
                .debug_tuple("BinOp")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::UnOp(arg0, arg1) => f.debug_tuple("UnOp").field(arg0).field(arg1).finish(),
        }
    }
}

impl From<IRNode> for f64 {
    fn from(val: IRNode) -> Self {
        match val {
            IRNode::Number(f) => f,
            _ => panic!("Failed to convert IRNode to f64"),
        }
    }
}