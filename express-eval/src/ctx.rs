use crate::ir::IRNode;
use express::{
    lang::ast::{Expression, Literal, Visit},
    types::{Callable, Function, Type},
};
use std::{collections::BTreeMap, rc::Rc};

type Namespace<T> = BTreeMap<String, T>;

/// A public interface for any Interpreter Context
pub trait InterpreterContext {
    /// Registers given function in the interpreter context
    fn register_function(&mut self, name: &str, exp_fn: Box<dyn Callable>);

    /// Registers given named constant in the interpreter context
    fn register_constant(&mut self, name: &str, exp_const: f64);

    fn find_function(&self, name: &str) -> Option<&Function>;

    fn find_constant(&self, name: &str) -> Option<f64>;
}

/// Holds evaluation context information such as functions
/// that implement `Callable` trait and named constants.
pub struct Context {
    pub ns_fn: Namespace<Function>,
    pub ns_const: Namespace<f64>,
    pub tmp_obj_lookup: BTreeMap<(String, Vec<Expression>), Function>,
}

impl Context {
    /// Constructs a new empty context
    pub fn new() -> Self {
        Self {
            ns_fn: Namespace::new(),
            ns_const: Namespace::new(),
            tmp_obj_lookup: BTreeMap::new(),
        }
    }
}

impl InterpreterContext for Context {
    /// Registers given function in the interpreter context
    fn register_function(&mut self, name: &str, exp_fn: Box<dyn Callable>) {
        self.ns_fn.insert(name.to_string(), Rc::from(exp_fn));
    }

    /// Registers given named constant in the interpreter context
    fn register_constant(&mut self, name: &str, exp_const: f64) {
        self.ns_const.insert(name.to_string(), exp_const);
    }

    fn find_function(&self, name: &str) -> Option<&Function> {
        self.ns_fn.get(name)
    }

    fn find_constant(&self, name: &str) -> Option<f64> {
        Some(*self.ns_const.get(name)?)
    }
}

/// Initializes object
// fn init_object(f: Rc<dyn Callable>, args: &[Type]) {
//     if f.should_be_created() {
//         f.init(args)
//     }
// }

/// Simplifies ast and produces new IRNode
fn reduce_ast_node(f: Rc<dyn Callable>, arguments: Vec<IRNode>) -> Result<IRNode, String> {
    if arguments.iter().any(|a| !matches!(a, IRNode::Value(_))) {
        return Ok(IRNode::Function(f.clone(), arguments));
    } else {
        let values: Box<[Type]> = arguments
            .into_iter()
            .map(|arg| match arg {
                IRNode::Value(t) => t,
                _ => unreachable!(),
            })
            .collect();
        if let Some(result) = f.call(&*values) {
            return Ok(IRNode::Value(result));
        }
        return Err(format!("Pure function with const arguments returned None"));
    }
}

/// Introducing dyn InterpreterContext will degrade performance
/// by inderection (vtable). While This visit is not important
/// later evaluation will suffer a perf hit since they would do
/// fn lookups through a vtable call.

impl Visit<Expression> for Context {
    type Returns = Result<IRNode, String>;

    fn visit_const(&self, cnst: Expression) -> Self::Returns {
        if let Expression::Const(c) = cnst {
            match c {
                Literal::Number(num) => return Ok(IRNode::Value(Type::Number(num))),
                Literal::Ident(id) => {
                    if let Some(val) = self.find_constant(id.as_str()) {
                        return Ok(IRNode::Value(Type::Number(val)));
                    } else {
                        return Ok(IRNode::Value(Type::String(id)));
                    }
                }
            };
        };
        Err(format!("Tried to visit const but it has other type"))
    }

    fn visit_fn(&self, xfn: Expression) -> Self::Returns {
        if let Expression::Function {
            name: Literal::Ident(name),
            args,
        } = xfn
        {
            // simplimies function arguments
            let mut arguments = Vec::with_capacity(args.len());
            // let object_key = (name, args.clone());
            for arg in args {
                arguments.push(self.visit_expr(arg)?);
            }

            if let Some(f) = self.find_function(name.as_str()) {
                if f.argcnt() != arguments.len() {
                    return Err(format!(
                        "Functions recieved unexpected number of arguments: {} ({} needed)",
                        arguments.len(),
                        f.argcnt()
                    ));
                }
                // self.tmp_obj_lookup.insert(object_key, *f);
                // init_object(*f, args.as_slice());

                // Try to simplify fn call
                if f.is_pure() {
                    return reduce_ast_node(f.clone(), arguments);
                } else {
                    return Ok(IRNode::Function(f.clone(), arguments));
                }
            }
            return Err(format!("Failed to find function with a name {}", name));
        }
        Err(format!("Tried to visit function but it has other type"))
    }

    fn visit_binop(&self, bin: Expression) -> Self::Returns {
        if let Expression::BinOp(lhs, rhs, op) = bin {
            let lhs = self.visit_expr(*lhs)?;
            let rhs = self.visit_expr(*rhs)?;
            return match (&lhs, &rhs) {
                (IRNode::Value(Type::Number(l)), IRNode::Value(Type::Number(r))) => {
                    Ok(IRNode::Value(Type::Number(op.eval(*l, *r))))
                }
                (IRNode::Value(l), IRNode::Value(r)) => {
                    return Err(format!(
                        "Cannot produce binary opertation between types {:?} and {:?}",
                        l, r
                    ))
                }
                _ => Ok(IRNode::BinOp(Box::new(lhs), Box::new(rhs), op)),
            };
        }
        Err(format!(
            "Tried to visit binary expression but it has other type"
        ))
    }

    fn visit_unop(&self, un: Expression) -> Self::Returns {
        if let Expression::UnOp(op, e) = un {
            let rhs = self.visit_expr(*e)?;
            if let IRNode::Value(Type::Number(rhs)) = rhs {
                return Ok(IRNode::Value(Type::Number(op.unary_eval(rhs))));
            }
            return Ok(IRNode::UnOp(Box::new(rhs), op));
        }
        Err(format!(
            "Tried to visit unary expression but it has other type"
        ))
    }

    fn visit_expr(&self, expr: Expression) -> Self::Returns {
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
    use express::lang::ast::Operation;
    use express::lang::parser::parse_expression;
    use express::xmacro::runtime_callable;
    use std::rc::Rc;

    #[runtime_callable]
    fn add_answer(val: f64) -> Option<f64> {
        Some(42.0 + val)
    }

    #[runtime_callable(pure)]
    fn succ(val: f64) -> Option<f64> {
        Some(val + 1.0)
    }

    #[runtime_callable(pure)]
    fn add(lhs: f64, rhs: f64) -> Option<f64> {
        Some(rhs + lhs)
    }

    #[runtime_callable(pure)]
    fn take_str(obj: String) -> Option<String> {
        Some(format!("New {} ", obj))
    }

    #[runtime_callable(pure)]
    fn repeat(astr: String, times: usize) -> Option<String> {
        Some(astr.repeat(times))
    }

    macro_rules! test_expr {
        ($expr: expr; $($cnst: expr => $cval: expr),*; $($fns: expr => $fval: expr),*) => {
            {
                let (_, expression) = parse_expression($expr).unwrap();
                // println!("\nEXPR: {}\n{:?}", $expr, expression);
                let mut ctx = Context::new();
                $( ctx.register_constant($cnst, $cval); )*
                $( ctx.register_function($fns, $fval); )*
                ctx.visit_expr(expression).unwrap()
            }
        };
    }

    #[test]
    pub fn test_const_inline() {
        let result = test_expr!("PI"; "PI" => 3.14;);
        assert_eq!(result, IRNode::Value(Type::Number(3.14)));
    }

    #[test]
    pub fn test_const_inline_add() {
        let result = test_expr!("PI + PI"; "PI" => 3.14;);
        assert_eq!(result, IRNode::Value(Type::Number(3.14 + 3.14)));
    }

    #[test]
    pub fn test_const_inline_paren() {
        let result = test_expr!("PI + (2 - 3)"; "PI" => 3.14;);
        assert_eq!(result, IRNode::Value(Type::Number(3.14 - 1.0)));
    }

    #[test]
    pub fn test_inline_paren() {
        let result = test_expr!("2 + 2 * TWO"; "TWO" => 2.0;);
        assert_eq!(result, IRNode::Value(Type::Number(6.0)));
    }

    #[test]
    pub fn test_const_inline_un() {
        let result = test_expr!("-Foo"; "Foo" => 1.0;);
        assert_eq!(result, IRNode::Value(Type::Number(-1.0)));
    }

    #[test]
    pub fn test_inline_un_expr() {
        let result = test_expr!("-Foo * 2 + (10**2)"; "Foo" => 1.0;);
        assert_eq!(
            result,
            IRNode::Value(Type::Number(-1.0 * 2.0 + (10.0f64.powf(2.0))))
        );
    }

    #[test]
    pub fn test_inline_fn_expr() {
        let result =
            test_expr!("-add_answer(1)"; "Foo" => 1.0; "add_answer" => Box::new(__add_answer));
        assert_eq!(
            result,
            IRNode::UnOp(
                Box::new(IRNode::Function(
                    Rc::new(__add_answer),
                    vec![IRNode::Value(Type::Number(1.0))]
                )),
                Operation::Minus,
            )
        );
    }

    #[test]
    pub fn test_inline_const_fn_expr() {
        let result = test_expr!("-add_answer(1) * PI + 2"; "PI" => 3.14; "add_answer" => Box::new(__add_answer));
        assert_eq!(
            result,
            IRNode::BinOp(
                Box::new(IRNode::BinOp(
                    Box::new(IRNode::UnOp(
                        Box::new(IRNode::Function(
                            Rc::new(__add_answer),
                            vec![IRNode::Value(Type::Number(1.0))]
                        )),
                        Operation::Minus
                    )),
                    Box::new(IRNode::Value(Type::Number(3.14))),
                    Operation::Times,
                )),
                Box::new(IRNode::Value(Type::Number(2.0))),
                Operation::Plus,
            )
        );
    }

    #[test]
    pub fn test_pure_function_optimization() {
        let result = test_expr!("succ(succ(succ(1)))"; ; "succ" => Box::new(__succ));
        assert_eq!(result, IRNode::Value(Type::Number(4.0)));
    }

    #[test]
    pub fn test_pure_complex_optimization() {
        let result = test_expr!("add_answer(succ(succ(succ(1)))**TWO)"; "TWO" => 2.0; "succ" => Box::new(__succ), "add_answer" => Box::new(__add_answer));
        assert_eq!(
            result,
            IRNode::Function(
                Rc::new(__add_answer),
                vec![IRNode::Value(Type::Number(16.0))]
            )
        );
    }

    #[test]
    pub fn test_all_pure_optimization() {
        let result = test_expr!("add(4, succ(succ(succ(1)))**TWO)"; "TWO" => 2.0; "succ" => Box::new(__succ), "add" => Box::new(__add));
        assert_eq!(result, IRNode::Value(20.0.into()));
    }

    #[test]
    pub fn test_all_str_optimization() {
        let result = test_expr!(
            "repeat(take_str(blah), succ(succ(succ(2))))";;
            "take_str" => Box::new(__take_str), "repeat" => Box::new(__repeat), "succ" => Box::new(__succ));
        assert_eq!(
            result,
            IRNode::Value(String::from("New blah New blah New blah New blah New blah ").into())
        );
    }
}
