use std::{collections::BTreeMap, rc::Rc};

use express::{
    lang::ast::{Expression, Literal, Visit},
    types::{Callable, Function},
};

use crate::ir::IRNode;

type Namespace<T> = BTreeMap<String, T>;

pub struct Context {
    pub ns_fn: Namespace<Function>,
    pub ns_const: Namespace<f64>,
}

impl Context {
    /// Constructs a new empty context
    fn new() -> Self {
        Self {
            ns_fn: Namespace::new(),
            ns_const: Namespace::new(),
        }
    }

    fn populate_prelude(&mut self) {
        todo!()
    }

    /// Registers given function in the interpreter context
    pub fn register_function(&mut self, name: &str, exp_fn: Rc<dyn Callable + Send + Sync>) {
        self.ns_fn.insert(name.to_string(), Function(exp_fn));
    }

    /// Registers given named constant in the interpreter context
    pub fn register_constant(&mut self, name: &str, exp_const: f64) {
        self.ns_const.insert(name.to_string(), exp_const);
    }

    pub fn find_function(&self, name: &str) -> Option<&Function> {
        self.ns_fn.get(name)
    }

    pub fn find_constant(&self, name: &str) -> Option<f64> {
        Some(*self.ns_const.get(name)?)
    }
}

impl Visit<Expression> for Context {
    type Returns = IRNode;

    fn visit_const(&self, cnst: Expression) -> Result<Self::Returns, String> {
        if let Expression::Const(c) = cnst {
            match c {
                Literal::Number(num) => return Ok(IRNode::Number(num)),
                Literal::Ident(id) => {
                    if let Some(val) = self.find_constant(id.as_str()) {
                        return Ok(IRNode::Number(val));
                    }
                    return Err(format!("Failed to resolve named constant: {}", id));
                }
            };
        };
        Err(format!("Tried to visit const but it has other type"))
    }

    fn visit_fn(&self, xfn: Expression) -> Result<Self::Returns, String> {
        if let Expression::Function {
            name: Literal::Ident(name),
            args,
        } = xfn
        {
            let mut arguments = Vec::with_capacity(args.len());
            for arg in args {
                arguments.push(self.visit_expr(arg)?);
            }
            if let Some(f) = self.find_function(name.as_str()) {
                return Ok(IRNode::Function(f.0.clone(), arguments));
            }
            return Err(format!("Failed to find function with a name {}", name));
        }
        Err(format!("Tried to visit function but it has other type"))
    }

    fn visit_binop(&self, bin: Expression) -> Result<Self::Returns, String> {
        if let Expression::BinOp(lhs, rhs, op) = bin {
            let lhs = self.visit_expr(*lhs)?;
            let rhs = self.visit_expr(*rhs)?;
            return match (&lhs, &rhs) {
                (IRNode::Number(l), IRNode::Number(r)) => Ok(IRNode::Number(op.eval(*l, *r))),
                _ => Ok(IRNode::BinOp(Box::new(lhs), Box::new(rhs), op)),
            };
        }
        Err(format!(
            "Tried to visit binary expression but it has other type"
        ))
    }

    fn visit_unop(&self, un: Expression) -> Result<Self::Returns, String> {
        if let Expression::UnOp(op, e) = un {
            let rhs = self.visit_expr(*e)?;
            if let IRNode::Number(rhs) = rhs {
                return Ok(IRNode::Number(op.unary_eval(rhs)));
            }
            return Ok(IRNode::UnOp(Box::new(rhs), op));
        }
        Err(format!(
            "Tried to visit unary expression but it has other type"
        ))
    }

    fn visit_expr(&self, expr: Expression) -> Result<Self::Returns, String> {
        match expr {
            Expression::Const(_) => self.visit_const(expr),
            Expression::Function { .. } => self.visit_fn(expr),
            Expression::BinOp(..) => self.visit_binop(expr),
            Expression::UnOp(..) => self.visit_unop(expr),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use express::lang::parser::parse_expression;
    type Ex = Expression;
    type Lt = Literal;

    #[test]
    pub fn test_const_inline() {
        let expression = Ex::Const(Lt::Ident("PI".to_string()));
        let mut ctx = Context::new();
        ctx.register_constant("PI", 3.14);
        let result = ctx.visit_expr(expression).unwrap();
        assert_eq!(result, IRNode::Number(3.14));
    }

    #[test]
    pub fn test_const_inline_add() {
        let (_, expression) = parse_expression("PI + PI").unwrap();
        let mut ctx = Context::new();
        ctx.register_constant("PI", 3.14);
        let result = ctx.visit_expr(expression).unwrap();
        assert_eq!(result, IRNode::Number(3.14 + 3.14));
    }

    #[test]
    pub fn test_const_inline_paren() {
        let (_, expression) = parse_expression("PI + (3 - 2)").unwrap();
        let mut ctx = Context::new();
        ctx.register_constant("PI", 3.14);
        let result = ctx.visit_expr(expression).unwrap();
        assert_eq!(result, IRNode::Number(3.14 - 1.0));
    }
}
