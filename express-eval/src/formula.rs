use crate::{ctx::Context, ir::IRNode};
use express::lang::{ast::Visit, parser::parse_expression};
use express::types::Type;
use std::rc::Rc;

/// This is a shared primitive. There might be a need of changing
/// this into some other smart ptr which is fiendly with concurent
/// execution.
pub type SharedFormula = Rc<Formula>;
pub type LinkFormula = Option<SharedFormula>;

#[derive(PartialEq, Debug)]
pub struct Formula {
    pub ast: IRNode,
    pub next: LinkFormula,
    pub parents: Vec<Rc<Formula>>,
    pub result: Option<Type>,
}

impl Iterator for Formula {
    type Item = Type;

    fn next(&mut self) -> Option<Self::Item> {
        self.eval()
    }
}

impl Formula {
    pub fn new(expression: &str, eval_ctx: &Context) -> Result<Self, String> {
        let (_, ast) = match parse_expression(expression) {
            Ok(it) => it,
            Err(err) => return Err(format!("Failed to parse expression. Reason: {}", err)),
        };
        Ok(Self {
            next: None,
            ast: eval_ctx.visit_expr(ast)?,
            parents: vec![],
            result: None,
        })
    }

    /// Evaluates formula and returns its result as __Option<Type>__
    pub fn eval(&self) -> Option<Type> {
        self.visit_expr(&self.ast)
    }
}

impl Visit<&IRNode> for Formula {
    type Returns = Option<Type>;

    // NOTE(iy): This call is unused because visit_expr
    // already handles extraction of a constant
    fn visit_const(&self, _: &IRNode) -> Self::Returns {
        unreachable!()
    }

    fn visit_fn(&self, _: &IRNode) -> Self::Returns {
        unreachable!()
    }

    fn visit_binop(&self, _: &IRNode) -> Self::Returns {
        unreachable!()
    }

    fn visit_unop(&self, _: &IRNode) -> Self::Returns {
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
            IRNode::Ref(_) => None,
        }
    }
}
