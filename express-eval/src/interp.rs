use crate::ctx::{Context, InterpreterContext};
use crate::formula::{Formula, SharedFormula};
use crate::ir::IRNode;
use express::lang::ast::Visit;
use express::types::Type;
use express::xmacro::use_library;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

type NamedExpression<'e> = (&'e str, &'e str);

/// This is the return type of any formula computation.
/// While formulas can handle any type from `express::types::Type`
/// as either input or output types, result of the final expression
/// node must be of type `IReturn`
//type IReturn = f64;

// NOTE(iy):
// On Interpreter optimizations
// | Resolve references (&name)
// | Inline reference AST if it is reduced to IRNode::Value
// | Find common partial AST inside each expression
// | => promote it to a new formula
// | => insert references to the new formula inplace
// | Init stateful functions (?)

/// Represents evaluation entry point. When supplied a new Context
/// it gets populated with std library contents automatically.
/// Before you instanciate the interpreter you must call `use_library!`
/// to populate it with your custom 3rd party library code if needed.
pub struct Interpreter {
    pub ctx: Context,
    pub active_nodes: Vec<String>,
    pub node_map: BTreeMap<String, SharedFormula>,
}

/// Loads functions and constants
/// from the standart library
fn load_prelude(ctx: &mut Context) {
    use_library! {
        context ctx;
        library express_std;
        constants {
            math::PI;
            math::EPS;
            math::TAU;
            math::LN2;
        }

        functions {
            math::log;
            math::ln;
            timeseries::ema;
            timeseries::jma;
            timeseries::ma;
            timeseries::malin;
            timeseries::twa;
        }
    }
}

impl Interpreter {
    /// Creates a new interpreter context from
    pub fn new(formulas: &[NamedExpression], mut context: Context) -> Result<Self, String> {
        load_prelude(&mut context);
        let mut node_map: BTreeMap<String, SharedFormula> = BTreeMap::new();
        let mut nodes = Vec::new();
        for (name, exp) in formulas {
            let formula = Formula::new(exp, &context)?;
            nodes.push((*name, formula.clone()));
            node_map.insert(name.to_string(), Rc::new(RefCell::new(formula)));
        }
        let mut intrp = Self {
            ctx: context,
            node_map,
            active_nodes: vec![],
        };

        intrp.build_dag(nodes.into_iter())?;

        Ok(intrp)
    }

    pub fn build_dag<'a, It>(&mut self, nodes: It) -> Result<(), String>
    where
        It: Iterator<Item = (&'a str, Formula)>,
    {
        for (name, f) in nodes.into_iter() {
            let fr = self.resolve_ref(f.ast)?;
            self.node_map.get(name).unwrap().borrow_mut().ast = fr;
        }

        if !self.active_nodes.is_empty() {
            Err(format!("There is no formula without a dependency"))
        } else {
            Ok(())
        }
        // fn dfs() {}
    }

    fn resolve_ref(&mut self, mut expr: IRNode) -> Result<IRNode, String> {
        match expr {
            IRNode::Value(_) => Ok(expr),
            IRNode::Function(_, ref mut args) => {
                for arg in args.iter_mut() {
                    *arg = self.resolve_ref(arg.clone())?;
                }
                Ok(expr)
            }
            IRNode::BinOp(ref mut lhs, ref mut rhs, _) => {
                **lhs = self.resolve_ref(*lhs.clone())?;
                **rhs = self.resolve_ref(*rhs.clone())?;
                Ok(expr)
            }
            IRNode::UnOp(ref mut rhs, _) => {
                **rhs = self.resolve_ref(*rhs.clone())?;
                Ok(expr)
            }
            IRNode::Ref(ref mut fref) => {
                if let Some(f) = self.node_map.get(fref.name.as_str()) {
                    fref.link_with(f.clone());
                    Ok(expr)
                } else {
                    Err(format!("Failed to find referant formula '{}'", fref.name))
                }
            }
        }
    }

    /// Evaluates formula
    pub fn eval(&self, formula_name: &str) -> Option<Type> {
        let formula = self.node_map.get(formula_name)?;
        self.visit_expr(&formula.borrow().ast)
    }

    pub fn eval_threaded(&self, th_num: usize) -> &[Option<Type>] {
        unimplemented!()
    }
}

/// Implements interator trait over interpreter.
/// The return value of the `next` is a Box ptr to
/// the slice of `Type`.
/// Why `Box<T>`? GATs at the moment are unstable
/// and the only way to use them is by swithing to
/// the nightly toolchain.
// impl Iterator for Interpreter {
//     /// GATs are unstable at the moment.
//     /// We can not write &'r Option<Type>
//     type Item = Box<[Option<Type>]>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         Some(self.formulas.iter().map(|(_, v)| self.eval(v)).collect())
//     }
// }

impl Visit<&IRNode> for Interpreter {
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

#[cfg(test)]
mod test {

    use super::*;
    use crate::ctx::InterpreterContext;
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
        let ctx = test_expr!(; "add" => Box::new(resolve_name!(add)));
        let i = Interpreter::new(&[("foo", "2 + add(12 - 2, add(1, 1))")], ctx).unwrap();
        let result: f64 = i.eval("foo").unwrap().into();
        assert_eq!(result, 14.0);
    }

    #[test]
    pub fn expr_with_std_call() {
        let intrp = Interpreter::new(&[("foo", "2+2*2+log(2,4)")], Context::new()).unwrap();
        let result: i64 = intrp.eval("foo").unwrap().into();
        assert_eq!(result, 8);
    }
}
