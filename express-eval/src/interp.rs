use crate::ctx::Context;
use crate::formula::Formula;
use crate::ir::IRNode;
use express::lang::ast::Visit;
use express::types::Type;

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
    pub ctx: Context,
    pub formulas: Vec<Formula>,
}

impl Interpreter {
    /// Creates a new interpreter context from
    pub fn new(formulas: &[NamedExpression], context: Context) -> Result<Self, String> {
        let mut fs = Vec::with_capacity(formulas.len());
        for (name, exp) in formulas {
            fs.push(Formula::new(name, exp, &context)?);
        }
        Ok(Self {
            ctx: context,
            formulas: fs,
        })
    }

    /// Evaluates interpreter context
    pub fn eval(&self, formula: &Formula) -> Option<Type> {
        self.visit_expr(&formula.ast)
    }
}

/// Implements interator trait over interpreter.
/// The return value of the `next` is a Box ptr to
/// the slice of `Type`.
/// Why `Box<T>`? GATs at the moment are unstable
/// and the only way to use them is by swithing to
/// the nightly toolchain.
impl Iterator for Interpreter {
    /// GATs are unstable at the moment.
    /// We can not write &'r Option<Type>
    type Item = Box<[Option<Type>]>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.formulas.iter().map(|f| self.eval(f)).collect())
    }
}

impl Visit<&IRNode> for Interpreter {
    type Returns = Option<Type>;

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
            // NOTE(iy): smelly part. We have to clone values.
            // Its ok for Number/TimeStep/Collection(it only clones ptr) but might be bad for
            // String.
            // FIXME: Possibly introduce currying at optimization level?
            IRNode::Value(n) => Some((*n).clone()),
            IRNode::Function(fn_obj, args) => {
                let mut resolved_args = Vec::with_capacity(args.len());
                // resolves arguments
                for arg in args {
                    resolved_args.push(self.visit_expr(arg)?);
                }
                Some(fn_obj.call(resolved_args.as_slice())?.into())
            }
            IRNode::BinOp(lhs, rhs, op) => {
                let lhs: f64 = self.visit_expr(lhs)?.into();
                let rhs: f64 = self.visit_expr(rhs)?.into();
                Some(Type::Number(op.eval(lhs, rhs)))
            }
            IRNode::UnOp(rhs, op) => {
                let rhs: f64 = self.visit_expr(rhs)?.into();
                Some(Type::Number(op.unary_eval(rhs)))
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use super::*;
    use express::types::{Callable, Type};
    use express::xmacro::{resolve_name, runtime_callable};

    #[runtime_callable]
    fn add(x: f64, y: f64) -> Option<f64> {
        Some(x + y)
    }

    macro_rules! test_expr {
        ($($cnst: expr => $cval: expr),*; $($fns: expr => $fval: expr),*) => {
            {
                let mut ctx = Context::new();
                $( ctx.register_constant($cnst, $cval); );*
                $( ctx.register_function($fns, $fval); );*
                ctx
            }
        };
    }

    #[test]
    pub fn simple_expression() {
        let ctx = test_expr!(; "add" => Rc::new(resolve_name!(add)));
        let i = Interpreter::new(&[("foo", "2 + add(12 - 2, add(1, 1))")], ctx).unwrap();
        let f = &i.formulas[0];
        let result: f64 = i.eval(f).unwrap().into();
        assert_eq!(result, 14.0);
    }
}
