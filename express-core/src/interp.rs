use crate::formula::Formula;
use std::collections::BTreeMap;
use types::{Callable, Function};

type Namespace<T> = BTreeMap<String, T>;
type NamedExpression<'e> = (&'e str, &'e str);

struct Interpreter {
    ctx_fn: Namespace<Function>,
    ctx_const: Namespace<f64>,
    formulas: Vec<Formula>,
}

impl Interpreter {
    /// Creates a new interpreter context from
    pub fn new(formulas: &[NamedExpression]) -> Result<Self, String> {
        let mut fs = Vec::new();
        for (name, exp) in formulas {
            fs.push(Formula::new(name, exp)?);
        }
        Ok(Self {
            ctx_fn: BTreeMap::new(),
            ctx_const: BTreeMap::new(),
            formulas: fs,
        })
    }

    /// Registers given function in the interpreter context
    pub fn register_function(&mut self, name: &str, exp_fn: Box<dyn Callable + Send + Sync>) {
        self.ctx_fn.insert(name.to_string(), Function(exp_fn));
    }

    /// Registers given named constant in the interpreter context
    pub fn register_constant(&mut self, name: &str, exp_const: f64) {
        self.ctx_const.insert(name.to_string(), exp_const);
    }

    /// Evaluates interpreter context
    pub fn eval() {
        unimplemented!()
    }

    /// optimizes ast
    pub fn optimize() {
        unimplemented!()
    }
}
