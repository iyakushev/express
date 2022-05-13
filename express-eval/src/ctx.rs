use std::{collections::BTreeMap, rc::Rc};

use express::{
    lang::ast::{Expression, Literal, Visit},
    types::{Callable, Function},
};

use crate::ir::IRNode;

type Namespace<T> = BTreeMap<String, T>;

#[derive(Debug)]
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

    macro_rules! test_expr {
        ($expr: expr; $($cnst: expr => $cval: expr),+; $($fns: expr => $fval: expr),*) => {
            {
                let (_, expression) = parse_expression($expr).unwrap();
                println!("\nEXPR: {}\n{:?}", $expr, expression);
                let mut ctx = Context::new();
                $( ctx.register_constant($cnst, $cval); );*
                ctx.visit_expr(expression).unwrap()
            }
        };
    }

    #[test]
    pub fn test_const_inline() {
        let result = test_expr!("PI"; "PI" => 3.14;);
        assert_eq!(result, IRNode::Number(3.14));
    }

    #[test]
    pub fn test_const_inline_add() {
        let result = test_expr!("PI + PI"; "PI" => 3.14;);
        assert_eq!(result, IRNode::Number(3.14 + 3.14));
    }

    #[test]
    pub fn test_const_inline_paren() {
        let result = test_expr!("PI + (2 - 3)"; "PI" => 3.14;);
        assert_eq!(result, IRNode::Number(3.14 - 1.0));
    }

    #[test]
    pub fn test_inline_paren() {
        let result = test_expr!("2 + 2 * TWO"; "TWO" => 2.0;);
        assert_eq!(result, IRNode::Number(6.0));
    }

    #[test]
    pub fn test_const_inline_un() {
        let result = test_expr!("-Foo"; "Foo" => 1.0;);
        assert_eq!(result, IRNode::Number(-1.0));
    }

    #[test]
    pub fn test_inline_un_expr() {
        let result = test_expr!("-Foo * 2 + (10**2)"; "Foo" => 1.0;);
        assert_eq!(result, IRNode::Number(-1.0 * 2.0 + (10.0f64.powf(2.0))));
    }
}
