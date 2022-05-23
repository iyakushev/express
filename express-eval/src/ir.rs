use crate::formula::{Formula, SharedFormula};
use express::{
    lang::ast::Operation,
    types::{Callable, Type},
};
use std::{fmt::Debug, rc::Rc};

#[derive(Debug, PartialEq)]
pub struct FormulaLink<Link> {
    name: String,
    link: Option<Link>,
}

impl<Link: Clone> FormulaLink<Link> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            link: None,
        }
    }

    pub fn is_resolved(&self) -> bool {
        self.link.is_some()
    }

    pub fn link_with(&mut self, formula: Link) {
        self.link = Some(formula.clone());
    }
}

pub enum IRNode {
    Value(Type),
    // NOTE(iy): Pointer primitive requires changes when adopting
    // a parallel execution model (Something like RWLock?).
    Ref(FormulaLink<SharedFormula>),
    Function(Rc<dyn Callable>, Vec<IRNode>),
    BinOp(Box<IRNode>, Box<IRNode>, Operation),
    UnOp(Box<IRNode>, Operation),
}

impl PartialEq for IRNode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Value(l0), Self::Value(r0)) => l0 == r0,
            (Self::Function(_, l1), Self::Function(_, r1)) => l1 == r1,
            (Self::BinOp(l0, l1, l2), Self::BinOp(r0, r1, r2)) => l0 == r0 && l1 == r1 && l2 == r2,
            (Self::UnOp(l0, l1), Self::UnOp(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Ref(l), Self::Ref(r)) => l == r,
            _ => false,
        }
    }
}

impl Debug for IRNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Value(arg0) => f.debug_tuple("Number").field(arg0).finish(),
            Self::Function(_, arg1) => f.debug_tuple("Function").field(arg1).finish(),
            Self::BinOp(arg0, arg1, arg2) => f
                .debug_tuple("BinOp")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::UnOp(arg0, arg1) => f.debug_tuple("UnOp").field(arg0).field(arg1).finish(),
            Self::Ref(r) => f.debug_tuple("Ref").field(r).finish(),
        }
    }
}

impl From<IRNode> for Type {
    fn from(val: IRNode) -> Self {
        match val {
            IRNode::Value(f) => f,
            _ => panic!("Failed to convert IRNode to Type"),
        }
    }
}
