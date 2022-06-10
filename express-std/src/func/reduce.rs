use express::types::{Callable, Function, InterpreterContext, Type};

struct Reduce {
    state: Type,
    func: Function,
}

// reduce(0, add, 1, 1)

impl Callable for Reduce {
    // fn signature: reduce(0, fn, arg*)
    // 1. f64 -- initial state
    // 2. expr -- an expression which will be called
    // 3. arg -- arguments to the expr(2)
    fn init(&mut self, args: &[Type], ctx: &dyn InterpreterContext) {
        self.state = args[0].clone();
        let fname: String = args[1].clone().into();
        if let Some(f) = ctx.find_function(fname.as_str()) {
            self.func = f.clone();
        } else {
            panic!("Failed to find function '{fname}' in the context");
        }
    }

    #[inline]
    fn name(&self) -> &str {
        "reduce"
    }

    #[inline]
    fn call(&mut self, args: &[Type]) -> Option<Type> {
        let args = &args[2..];
        let mut f = self.func.borrow_mut();
        if args.len() != f.argcnt() {
            panic!(
                "Function {} recieved {} arguments, but expects {}",
                f.name(),
                args.len(),
                f.argcnt()
            );
        }
        self.state = f.call(&args[2..])?;
        Some(self.state.clone())
    }

    #[inline]
    fn argcnt(&self) -> usize {
        3
    }
}
