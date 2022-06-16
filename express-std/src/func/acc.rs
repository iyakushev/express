use express::prelude::*;

/// This is how **you define a constructor**:
/// Note that it is also handy to get a gist of function signature.
/// This signature will be employed in the real `call` method as `args: &[Type]`.
/// You can think of it as currying. But on each call you recieve all arguments.
/// Like yeah, currying in compile time is object construction. How. Cool. Is. That?!
#[runtime_callable(constant)]
pub fn acc(init: f64, _f: &Type) -> Accumulate {
    Accumulate { acc: init }
}

/// The real function state
pub struct Accumulate {
    acc: f64,
}

impl Callable for Accumulate {
    #[inline(always)]
    fn name(&self) -> &'static str {
        "Accum"
    }

    #[inline]
    fn call(&mut self, args: &[Type]) -> Option<Type> {
        let arg = args[1].clone();
        match arg {
            Type::Number(num) => self.acc += num,
            Type::None => return None,
            t @ _ => panic!("Acc function recieved unsupported type: {t}"),
        }
        Some(self.acc.into())
    }

    #[inline(always)]
    fn argcnt(&self) -> usize {
        2
    }
}
