use crate::ctx::Context;
use crate::formula::Formula;
use std::collections::BTreeMap;
use std::rc::Rc;

type NamedExpression<'e> = (&'e str, &'e str);

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
