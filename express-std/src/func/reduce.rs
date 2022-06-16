use express::prelude::*;

// TODO: Need to support variadic arguments
// Need to support implicit ctx lookup

// #[runtime_callable(constant)]
// fn reduce(state: f64, func: String) -> Reduce {
//     Reduce { state, func }
// }

struct Reduce {
    state: f64,
    func: Function,
}
// fn signature: reduce(0, fn, arg*)
// 1. f64 -- initial state
// 2. expr -- an expression which will be called
// 3. arg -- arguments to the expr(2)
// fn init(args: &[Type], ctx: &dyn InterpreterContext) {
//   self.state = args[0].clone().into();
//   let fname: String = args[1].clone().into();
//   if let Some(f) = ctx.find_function(fname.as_str()) {
//       self.func = f.clone();
//   } else {
//       panic!("Failed to find function '{fname}' in the context");
//   }
//}
// reduce(0, add, 1, 1)

impl Callable for Reduce {
    #[inline]
    fn name(&self) -> &'static str {
        "reduce"
    }

    #[inline]
    fn call(&mut self, args: &[Type]) -> Option<Type> {
        let args = &args[2..];
        if args.len() != self.func.argcnt() {
            panic!(
                "Function {} recieved {} arguments, but expects {}",
                self.func.name(),
                args.len(),
                self.func.argcnt()
            );
        }
        let state = self.func.call(&args[2..])?;
        self.state = state.clone().into();
        Some(state)
    }

    #[inline]
    fn argcnt(&self) -> usize {
        3
    }
}
