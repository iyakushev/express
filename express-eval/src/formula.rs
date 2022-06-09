use crate::{ctx::Context, ir::IRNode};
use express::lang::{ast::Visit, parser::parse_expression};
use express::types::Type;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

/// This is a shared primitive. There might be a need of changing
/// this into some other smart ptr which is fiendly with concurent
/// execution.
//pub type SharedFormula = Rc<RefCell<Formula>>;
pub type SharedFormula = Rc<RefCell<Formula>>;
pub type RefFormula = Weak<RefCell<Formula>>;

#[derive(PartialEq, Clone)]
pub struct Formula {
    pub name: String, // GATs!?! WHERE ARE MY GATS!?
    pub ast: IRNode,
    pub children: Vec<SharedFormula>, // this should probably be a RefFormula
    pub parents: Vec<SharedFormula>,
    pub result: Option<Type>,
}

impl Debug for Formula {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Formula")
            .field("name", &self.name)
            .field("ast", &self.ast)
            .field(
                "children",
                &self
                    .children
                    .iter()
                    .map(|ch| ch.borrow().name.clone())
                    .collect::<Vec<String>>(),
            )
            .field(
                "parents",
                &self
                    .children
                    .iter()
                    .map(|p| p.borrow().name.clone())
                    .collect::<Vec<String>>(),
            )
            .field("result", &self.result)
            .finish()
    }
}

impl Formula {
    pub fn new(name: &str, expression: &str, eval_ctx: &Context) -> Result<Self, String> {
        let (_, ast) = match parse_expression(expression) {
            Ok(it) => it,
            Err(err) => return Err(format!("Failed to parse expression. Reason: {}", err)),
        };
        Ok(Self {
            name: name.to_string(),
            children: vec![],
            ast: eval_ctx.visit_expr(ast)?,
            parents: vec![],
            result: None,
        })
    }

    /// Consumes formula and creates SharedFormula
    pub fn make_shared(self) -> SharedFormula {
        Rc::new(RefCell::new(self))
    }

    /// Evaluates formula and returns its result as __Option<Type>__
    pub fn eval(&self) -> Option<Type> {
        self.visit_expr(&self.ast)
    }

    /// Stores evaluation result inside .result field
    pub fn eval_inplace(&mut self) {
        self.result = self.visit_expr(&self.ast);
    }

    /// inlines reference
    pub fn inline_ref(&mut self, rf: SharedFormula) {
        let name = rf.borrow().name.clone();
        let target_ref = rf.borrow().ast.clone();
        self.ast = self.__inline_ref(self.ast.clone(), &name, target_ref);
        // remove reference from parent
        self.parents.retain(|el| *el != rf);
    }

    fn __inline_ref(&mut self, mut expr: IRNode, t_name: &str, trgt: IRNode) -> IRNode {
        match expr {
            IRNode::Value(_) => expr,
            IRNode::Ref(ref rf) => {
                if let IRNode::Function(..) = trgt {
                    if rf.name == t_name {
                        return trgt.clone();
                    } else {
                        return expr;
                    }
                } else {
                    unreachable!()
                }
            }
            IRNode::Function(_, ref mut args) => {
                for arg in args.iter_mut() {
                    *arg = self.__inline_ref(arg.clone(), t_name, trgt.clone());
                }
                expr
            }
            IRNode::BinOp(ref mut lhs, ref mut rhs, _) => {
                **rhs = self.__inline_ref(*rhs.clone(), t_name, trgt.clone());
                **lhs = self.__inline_ref(*lhs.clone(), t_name, trgt.clone());
                expr
            }
            IRNode::UnOp(ref mut lhs, _) => {
                **lhs = self.__inline_ref(*lhs.clone(), t_name, trgt);
                expr
            }
        }
    }

    pub fn resolve_ref(
        &mut self,
        mut expr: IRNode,
        node_map: &BTreeMap<String, SharedFormula>,
    ) -> Result<IRNode, String> {
        match expr {
            IRNode::Value(_) => Ok(expr),
            IRNode::Function(_, ref mut args) => {
                for arg in args.iter_mut() {
                    *arg = self.resolve_ref(arg.clone(), node_map)?;
                }
                Ok(expr)
            }
            IRNode::BinOp(ref mut lhs, ref mut rhs, _) => {
                **lhs = self.resolve_ref(*lhs.clone(), node_map)?;
                **rhs = self.resolve_ref(*rhs.clone(), node_map)?;
                Ok(expr)
            }
            IRNode::UnOp(ref mut rhs, _) => {
                **rhs = self.resolve_ref(*rhs.clone(), node_map)?;
                Ok(expr)
            }
            IRNode::Ref(ref mut fref) => {
                if let Some(f) = node_map.get(fref.name.as_str()) {
                    // OPTIMIZATION: inline const ast
                    if let IRNode::Value(val) = &f.borrow().ast {
                        return Ok(IRNode::Value(val.clone()));
                    } else {
                        fref.link_with(f);
                        self.parents.push(f.clone());
                        return Ok(expr);
                    }
                } else {
                    Err(format!("Failed to find referant formula '{}'", fref.name))
                }
            }
        }
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
            IRNode::Ref(formula) => formula.link().as_deref()?.borrow().result.clone(),
        }
    }
}
