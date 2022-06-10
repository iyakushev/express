use crate::ctx::{Context, InterpreterContext};
use crate::formula::{Formula, SharedFormula};
use crate::ir::{FormulaLink, IRNode};
use express::lang::ast::Visit;
use express::types::Type;
use express::xmacro::use_library;
use std::cell::Ref;
use std::collections::{BTreeMap, BTreeSet};
use std::mem::swap;

type NamedExpression<'e> = (&'e str, &'e str);

/// This is the return type of any formula computation.
/// While formulas can handle any type from `express::types::Type`
/// as either input or output types, result of the final expression
/// node must be of type `IReturn`
//type IReturn = f64;

// NOTE(iy):
// On Interpreter optimizations
// |[x] Resolve references (&name)
// |[x] Build DAG with dfs check
// |[x] Inline reference AST if it is reduced to IRNode::Value
// |[ ] Init stateful functions (?)
// |[+-] Find common partial AST inside each expression
// |[x]     => promote it to a new formula
// |[x]     => insert references to the new formula inplace

/// Represents evaluation entry point. When supplied a new Context
/// it gets populated with std library contents automatically.
/// Before you instanciate the interpreter you must call `use_library!`
/// to populate it with your custom 3rd party library code if needed.
pub struct Interpreter {
    pub ctx: Context,
    pub root_nodes: Vec<SharedFormula>,
    pub node_map: BTreeMap<String, SharedFormula>,
}

/// Assignes next node to a collection of parents
fn link_child_with_parents(refs: &mut [SharedFormula], next: SharedFormula) {
    refs.iter()
        .for_each(|fref| fref.borrow_mut().children.push(next.clone()));
}

/// Good ol' Depth-first search
fn dfs(
    node: Ref<Formula>,
    known: &mut BTreeSet<String>, // FIXME remove unnecessary allocs for String
    stack_trace: &mut BTreeSet<String>,
) -> Result<(), String> {
    for child in &node.children {
        let child = child.borrow();
        let name = child.name.clone();
        if stack_trace.contains(child.name.as_str()) {
            return Err(format!(
                "Encountered a dependancy cycle! Node '{}' already has a dependency '{}'",
                node.name, child.name
            ));
        } else if !known.contains(name.as_str()) {
            stack_trace.insert(name);
            return dfs(child, known, stack_trace);
        }
    }
    stack_trace.remove(node.name.as_str());
    known.insert(node.name.clone());
    Ok(())
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
            math::common::max;
            math::common::min;
            timeseries::ema;
            timeseries::jma;
            timeseries::ma;
            timeseries::malin;
            timeseries::twa;
        }
    }
}

pub struct IntrerpterIt {
    interpreter: Interpreter, // &'i Interpreter but GATS!!
    children_buf: Vec<SharedFormula>,
}

impl Iterator for IntrerpterIt {
    type Item = BTreeMap<String, Option<Type>>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.interpreter.compute_pass(&mut self.children_buf);
        if result.iter().all(|(_, val)| val.is_none()) {
            None
        } else {
            Some(result)
        }
    }
}

impl IntoIterator for Interpreter {
    type Item = BTreeMap<String, Option<Type>>;

    type IntoIter = IntrerpterIt;

    fn into_iter(self) -> Self::IntoIter {
        IntrerpterIt {
            interpreter: self,
            children_buf: Vec::with_capacity(10),
        }
    }
}

// FOCKING GATS!!!!!!!!!!!!!!!!!!!!!!!
// impl IntoIterator for Interpreter {
//     type Item = BTreeMap<String, Option<Type>>;
//
//     type IntoIter<'i> = InterpreterIt<'i>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         IntrerpterIt {
//             interpreter: &self,
//             children_buf: Vec::with_capacity(10),
//         }
//     }
// }

impl Interpreter {
    /// Creates a new interpreter context from
    pub fn new(formulas: &[NamedExpression], mut context: Context) -> Result<Self, String> {
        // Load standard library
        load_prelude(&mut context);
        // TODO: optimization -> Make DAGbld struct that builds dag and holds node_map
        // since it would be unused after the DAG has been created
        let mut node_map: BTreeMap<String, SharedFormula> = BTreeMap::new();
        let mut nodes = Vec::new();
        for (name, exp) in formulas {
            let formula = Formula::new(name, exp, &context)?;
            nodes.push((name.to_string(), formula.clone()));
            node_map.insert(name.to_string(), formula.make_shared());
        }
        let mut intrp = Self {
            ctx: context,
            node_map,
            root_nodes: vec![],
        };

        intrp.build_dag(nodes.into_iter())?;

        Ok(intrp)
    }

    /// Creates a Direct Acyclic Graph for the stage execution.
    /// Refernces introduce dependencies and therefore they should be
    /// managed in a tree-flow fashion.
    fn build_dag<It>(&mut self, nodes: It) -> Result<(), String>
    where
        It: Iterator<Item = (String, Formula)>,
    {
        // order matters.
        // Unused links will be resolved in a reverse order
        let mut unused = Vec::new();
        for (name, _) in nodes.into_iter() {
            let fnode = self.node_map.get(&name).unwrap().clone();
            let mut fnode_inner = fnode.borrow_mut();

            // This would ensure that previous formula functions gets referenced
            self.manage_references(&mut fnode_inner, &mut unused)?;

            if fnode_inner.parents.is_empty() {
                self.root_nodes.push(fnode.clone());
            } else {
                link_child_with_parents(fnode_inner.parents.as_mut_slice(), fnode.clone());
            }
        }

        self.remove_redundant_references(unused.as_slice())?;
        self.assert_dag_has_no_cycles()?;
        self.opt_const_nodes();

        if self.root_nodes.is_empty() {
            Err(format!("Root nodes are empty. Execution graph is invalid"))
        } else {
            Ok(())
        }
    }

    fn manage_references(
        &mut self,
        formula: &mut Formula,
        unused: &mut Vec<String>,
    ) -> Result<(), String> {
        let mut ir = formula.ast.clone();
        // optimization: Incapsulate repeating functions in a separate formula
        ir = self._find_dup_fns(unused, ir.clone());
        // resolve references (links everything together)
        formula.ast = formula.resolve_ref(ir.clone(), &self.node_map)?;
        Ok(())
    }

    fn mangle_fname(node: &IRNode) -> String {
        format!("__{}", node)
    }

    fn remove_redundant_references(&mut self, unused: &[String]) -> Result<(), String> {
        for name in unused.iter() {
            if let Some(shared_f) = self.node_map.remove(name) {
                if shared_f.borrow().children.len() != 1 {
                    return Err(
                        format!("Failed to inline redundant reference &{}. It has {} children which makes it valid",
                                name,
                                shared_f.borrow().children.len()));
                }

                let ref_origin = shared_f.borrow_mut().children.pop().unwrap();
                ref_origin.borrow_mut().inline_ref(shared_f);

                let refast = ref_origin.borrow().ast.clone();
                let mut reforigin_mut = ref_origin.borrow_mut();
                reforigin_mut.ast = reforigin_mut.resolve_ref(refast, &self.node_map)?;
            }
        }

        // and assign new starting nodes
        for v in self.node_map.values() {
            if v.borrow().parents.is_empty() {
                self.root_nodes.push(v.clone());
            }
        }

        Ok(())
    }

    /// Incapsulates same partial tree nodes in a different formula.
    /// This optimization allows the compiler to initialize stateful
    /// functions only once and later compute them seperately to reuse
    /// their result.
    fn _find_dup_fns(&mut self, unused: &mut Vec<String>, mut expr: IRNode) -> IRNode {
        let fname = Interpreter::mangle_fname(&expr);
        match expr {
            IRNode::Value(_) => expr,
            IRNode::Ref(ref mut rf) => {
                if rf.count() <= 1 && rf.link().is_some() {
                    self.node_map.remove(&rf.name);
                    let f = rf.link().unwrap().clone();
                    drop(rf);
                    let f = f.borrow_mut().ast.clone();
                    return f;
                }
                expr
            }
            IRNode::Function(ref func, ref mut args) => {
                if func.is_pure() {
                    return expr;
                }
                // TODO: add the same optimization for arguments
                for arg in args.iter_mut() {
                    *arg = self._find_dup_fns(unused, arg.clone());
                }

                // mangle formula name and check its presents
                if let Some(val) = self.node_map.get(&fname) {
                    unused.retain(|el| el != &fname);
                    let mut link = FormulaLink::new(&fname);
                    link.link_with(val);

                    return IRNode::Ref(link);
                } else {
                    // create formula
                    let f = Formula {
                        ast: IRNode::Function(func.clone(), args.clone()),
                        children: vec![],
                        parents: vec![],
                        name: fname.clone(),
                        result: None,
                    };

                    let shared_f = f.make_shared();

                    let mut link = FormulaLink::new(&fname);
                    link.link_with(&shared_f);

                    // and record it
                    self.node_map.insert(fname.clone(), shared_f.clone());
                    unused.push(fname);
                    let mut shared = shared_f.borrow_mut();
                    let ast = shared.ast.clone();
                    // Resolve additional references inside of arguments
                    shared.ast = shared.resolve_ref(ast, &self.node_map).unwrap();
                    link_child_with_parents(shared.parents.as_mut_slice(), shared_f.clone());

                    return IRNode::Ref(link);
                }
            }
            IRNode::BinOp(ref mut lhs, ref mut rhs, _) => {
                **lhs = self._find_dup_fns(unused, *lhs.clone());
                **rhs = self._find_dup_fns(unused, *rhs.clone());
                expr
            }
            IRNode::UnOp(ref mut lhs, _) => {
                **lhs = self._find_dup_fns(unused, *lhs.clone());
                expr
            }
        }
    }

    /// Inline const result evaluation
    fn opt_const_nodes(&self) {
        for (_, f) in &self.node_map {
            let mut formula = f.borrow_mut();
            if let Some(val) = self._opt_const_helper(&formula.ast) {
                formula.ast = IRNode::Value(val);
            }
        }
    }

    fn _opt_const_helper(&self, expr: &IRNode) -> Option<Type> {
        match expr {
            // NOTE(iy): smelly part. We have to clone values.
            // Its ok for Number/TimeStep/Collection(it only clones ptr) but might be bad for
            // String.
            IRNode::Value(n) => Some((*n).clone()),
            IRNode::Function(fn_obj, args) => {
                if !fn_obj.is_pure() {
                    return None;
                }
                let mut resolved_args = Vec::with_capacity(args.len());

                // resolves arguments
                for arg in args {
                    resolved_args.push(self._opt_const_helper(arg)?);
                }
                Some(fn_obj.call(resolved_args.as_slice())?.into())
            }
            IRNode::BinOp(lhs, rhs, op) => {
                let lhs: f64 = self._opt_const_helper(lhs)?.into();
                let rhs: f64 = self._opt_const_helper(rhs)?.into();
                Some(Type::Number(op.eval(lhs, rhs)))
            }
            IRNode::UnOp(rhs, op) => {
                let rhs: f64 = self._opt_const_helper(rhs)?.into();
                Some(Type::Number(op.unary_eval(rhs)))
            }
            IRNode::Ref(formula) => formula.link().as_deref()?.borrow().result.clone(),
        }
    }

    fn assert_dag_has_no_cycles(&self) -> Result<(), String> {
        let mut known = BTreeSet::new();
        for (name, formula) in &self.node_map {
            let mut stack_trace = BTreeSet::new();
            known.insert(name.to_string());
            stack_trace.insert(name.to_string());
            dfs(formula.borrow(), &mut known, &mut stack_trace)?;
        }
        Ok(())
    }

    /// Evaluates formula
    pub fn eval(&self, formula: Ref<Formula>) -> Option<Type> {
        self.visit_expr(&formula.ast)
    }

    /// calculates a single iteration of the eval loop
    pub fn compute_pass(
        &self,
        children_buf: &mut Vec<SharedFormula>,
    ) -> BTreeMap<String, Option<Type>> {
        let mut active_nodes = self.root_nodes.clone();
        let mut results = BTreeMap::new();
        while !active_nodes.is_empty() {
            for node in &active_nodes {
                let mut formula = node.borrow_mut();
                formula.eval_inplace();
                children_buf.extend(formula.children.clone());
                if formula.children.is_empty() && !results.contains_key(&formula.name) {
                    results.insert(formula.name.clone(), formula.result.clone());
                }
            }
            swap(&mut active_nodes, children_buf);
            children_buf.clear();
        }
        results
    }

    pub fn _eval_threaded(&self, _th_num: usize) -> &[Option<Type>] {
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
            IRNode::Ref(formula) => formula.link().as_deref()?.borrow().result.clone(),
        }
    }
}

#[cfg(test)]
mod test {

    use std::rc::Rc;

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
        let f = i.node_map.get("foo").unwrap();
        let result: f64 = i.eval(f.borrow()).unwrap().into();
        assert_eq!(result, 14.0);
    }

    #[test]
    pub fn expr_with_std_call() {
        let intrp = Interpreter::new(&[("foo", "2+2*2+log(2,4)")], Context::new()).unwrap();
        let f = intrp.node_map.get("foo").unwrap();
        let result: i64 = intrp.eval(f.borrow()).unwrap().into();
        assert_eq!(result, 8);
    }

    #[test]
    pub fn expr_with_ref_call() {
        let mut ctx = Context::new();
        ctx.register_function("add", Box::new(__add));
        let intrp =
            Interpreter::new(&[("foo", "2+2*2+add(2,4)"), ("bar", "&foo * 2")], ctx).unwrap();
        let f = intrp.node_map.get("foo").unwrap();
        let result: i64 = intrp.eval(f.borrow()).unwrap().into();
        assert_eq!(result, 12);
        assert!(!f.borrow().children.is_empty());
        let next_from_root = intrp.node_map.get("bar").unwrap().clone();
        assert!(Rc::ptr_eq(&next_from_root, &f.borrow().children[0]));
        assert!(next_from_root.borrow().children.is_empty());
    }

    #[test]
    fn expr_opt_inline_const() {
        let intrp = Interpreter::new(
            &[("far", "2 + 2 * 2 + log(2, 4)"), ("bor", "&far * 2")],
            Context::new(),
        )
        .unwrap();
        assert!(intrp.root_nodes[0].borrow().children.is_empty());
        let f = intrp.node_map.get("bor").unwrap().borrow();
        assert_eq!(f.ast, IRNode::Value(Type::Number(16.0)));
    }

    #[test]
    fn expr_opt_const_2nd_pass() {
        let mut ctx = Context::new();
        ctx.register_function("add", Box::new(__add));
        let intrp = Interpreter::new(
            &[
                ("foo", "2 + 2 * 2 + log(2, 4)"),
                ("bar", "log(2, &foo)"),
                ("fuz", "add(2, &foo)"), // can't be optmizied
            ],
            ctx,
        )
        .unwrap();
        assert!(intrp.root_nodes[0].borrow().children.is_empty());
        let f = intrp.node_map.get("bar").unwrap().borrow();
        assert_eq!(f.ast, IRNode::Value(Type::Number(3.0)));
        let f = intrp.node_map.get("fuz").unwrap().borrow();
        assert_eq!(
            f.ast,
            IRNode::Function(
                Rc::new(__add),
                vec![
                    IRNode::Value(Type::Number(2.0)),
                    IRNode::Value(Type::Number(8.0))
                ]
            )
        );
    }

    #[test]
    pub fn expr_with_simple_cyclic_ref() {
        let intrp = Interpreter::new(
            &[("foo", "11 + &bary"), ("bary", "&foo + 11")],
            Context::new(),
        );
        assert!(matches!(intrp, Err(_)));
    }

    #[test]
    pub fn expr_with_nested_call() {
        let mut ctx = Context::new();
        ctx.register_function("add", Box::new(__add));
        let intrp = Interpreter::new(&[("foo", "add(1, add(1, add(1, add(1, 1))))")], ctx).unwrap();
        assert!(!intrp.node_map.is_empty());
        // println!("{:?}", intrp.node_map);
    }

    #[test]
    pub fn expr_with_repeating_calls() {
        let mut ctx = Context::new();
        ctx.register_function("add", Box::new(__add));
        let intrp = Interpreter::new(
            &[
                ("f1", "11 + add(1, 1)"),
                ("f2", "2 * add(1, 1) + add(1, 1)"),
            ],
            ctx,
        )
        .unwrap();
        assert_eq!(intrp.node_map.len(), 3);
    }

    #[test]
    pub fn test_compute_pass() {
        let mut ctx = Context::new();
        ctx.register_function("add", Box::new(__add));
        let intrp = Interpreter::new(
            &[
                ("f1", "11 + add(1, 1)"),
                ("f2", "2 * add(1, 1) + add(1, 1)"),
            ],
            ctx,
        )
        .unwrap();
        let result = intrp.compute_pass(&mut intrp.root_nodes.clone());
        assert!(!result.is_empty());
        assert_eq!(result["f1"], Some(Type::Number(13.0)));
        assert_eq!(result["f2"], Some(Type::Number(6.0)));
    }

    #[test]
    pub fn test_compute_pass_single_end() {
        let mut ctx = Context::new();
        ctx.register_function("add", Box::new(__add));
        let intrp = Interpreter::new(
            &[
                ("f1", "11 + add(1, 1)"),
                ("f2", "2 * add(1, 1) + add(1, 1)"),
                ("f3", "&f1 + &f2"),
            ],
            ctx,
        )
        .unwrap();
        let result = intrp.compute_pass(&mut intrp.root_nodes.clone());
        assert!(!result.is_empty());
        assert_eq!(result.len(), 1);
        assert_eq!(result["f3"], Some(Type::Number(19.0)));
    }

    #[test]
    pub fn test_compute_pass_iterator() {
        let mut ctx = Context::new();
        ctx.register_function("add", Box::new(__add));
        let intrp = Interpreter::new(
            &[
                ("f1", "11 + add(1, 1)"),
                ("f2", "2 * add(1, 1) + add(1, 1)"),
                ("f3", "&f1 + &f2"),
            ],
            ctx,
        )
        .unwrap();
        let mut iit = intrp.into_iter();
        let result = iit.next().unwrap();
        assert!(!result.is_empty());
        // for result in intrp  <-- results in an inf loop, since functions cant return None
    }
}
