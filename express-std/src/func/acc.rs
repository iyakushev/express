use express::types::{Callable, Type};

struct Accumulate {
    acc: f64,
}

impl Default for Accumulate {
    fn default() -> Self {
        Self {
            acc: Default::default(),
        }
    }
}

impl Callable for Accumulate {
    fn init(args: &[Type], _: &dyn express::types::InterpreterContext) -> Self {
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
