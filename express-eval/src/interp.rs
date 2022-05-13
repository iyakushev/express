use crate::ctx::Context;
use crate::formula::Formula;
use crate::ir::IRNode;
use express::lang::ast::Visit;
use std::rc::Rc;

type NamedExpression<'e> = (&'e str, &'e str);

/// This is the return type of any formula computation.
/// While formulas can handle any type from `express::types::Type`
/// as either input or output types, result of the final expression
/// node must be of type `IReturn`
type IReturn = f64;

macro_rules! include_std {
    ($obj: expr, $m: path , $name: ident) => {
        let strct = $m::$name::$name;
        $obj.ctx_fn.insert($name, strct);
    };
}
pub struct Interpreter {
    pub ctx: Rc<Context>,
    pub formulas: Vec<Formula>,
}

impl Interpreter {
    /// Creates a new interpreter context from
    pub fn new(formulas: &[NamedExpression], context: Context) -> Result<Self, String> {
        let mut fs = Vec::with_capacity(formulas.len());
        let ctx = Rc::new(context);
        for (name, exp) in formulas {
            fs.push(Formula::new(name, exp, ctx.clone())?);
        }
        Ok(Self { ctx, formulas: fs })
    }

    /// Evaluates interpreter context
    pub fn eval() {
        unimplemented!()
    }
}

impl Visit<&IRNode> for Interpreter {
    type Returns = Option<IReturn>;

    // NOTE(iy): This call is unused because visit_expr
    // already handles extraction of a constant
    fn visit_const(&self, cnst: &IRNode) -> Self::Returns {
        unreachable!()
    }

    fn visit_fn(&self, xfn: &IRNode) -> Self::Returns {
        unreachable!()
    }

    fn visit_binop(&self, bin: &IRNode) -> Self::Returns {
        unreachable!()
    }

    fn visit_unop(&self, un: &IRNode) -> Self::Returns {
        unreachable!()
    }

    fn visit_expr(&self, expr: &IRNode) -> Self::Returns {
        match expr {
            IRNode::Number(n) => Some(*n),
            // IRNode::Function(fn_obj, args) => {
            //     for arg in args {
            //         self.visit_expr(arg)?;
            //     }
            // }
            IRNode::BinOp(lhs, rhs, op) => {
                Some(op.eval(self.visit_expr(lhs)?, self.visit_expr(rhs)?))
            }
            IRNode::UnOp(rhs, op) => Some(op.unary_eval(self.visit_expr(&*rhs)?)),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use express::lang::parser::parse_expression;
    use express::xmacro::runtime_callable;

    #[runtime_callable]
    fn add(x: f64, y: f64) -> Option<f64> {
        Some(x + y)
    }

    macro_rules! test_expr {
        ($expr: expr; $($cnst: expr => $cval: expr),*; $($fns: expr => $fval: expr),*) => {
            {
                let (_, expression) = parse_expression($expr).unwrap();
                println!("\nEXPR: {}\n{:?}", $expr, expression);
                let mut ctx = Context::new();
                $( ctx.register_constant($cnst, $cval); );*
                // ctx.visit_expr(expression).unwrap()
            }
        };
    }

    #[test]
    pub fn simple_expression() {
        let expr = test_expr!("2 + add(12 - 2, add(1, 1))"; ;);
    }
}
