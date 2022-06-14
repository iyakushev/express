use express::{
    types::{Callable, InterpreterContext, Type},
    xmacro::runtime_callable,
};

/// This is how **you define a constructor**:
/// Note that it is also handy to get a gist of function signature.
/// This signature will be employed in the real `call` method as `args: &[Type]`.
/// You can think of it as currying. But on each call you recieve all arguments.
/// Like yeah, currying in compile time is object construction. How. Cool. Is. That?!
#[runtime_callable(pure)]
pub fn acc(init: f64, _f: &Type) -> Accumulate {
    Accumulate { acc: init }
}

/// The real function state
pub struct Accumulate {
    acc: f64,
}

impl Callable for Accumulate {
    fn init(&self, args: &[Type], _: &dyn express::types::InterpreterContext) -> Self {
        Self {
            acc: args[0].clone().into(),
        }
    }

    fn name(&self) -> &'static str {
        "acc"
    }

    fn call(&mut self, args: &[Type]) -> Option<Type> {
        let arg = args[1].clone();
        match arg {
            Type::Number(num) => self.acc += num,
            Type::None => return None,
            t @ _ => panic!("Acc function recieved unsupported type: {t}"),
        }
        Some(self.acc.into())
    }

    fn argcnt(&self) -> usize {
        2
    }
}
