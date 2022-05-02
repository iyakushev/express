pub enum Type {
    Number(f64),
    String(String),
    Function(Function),
    Collection(Box<[Type]>),
    TimeStep(TimeStep),
}

macro_rules! bijection {
    ($expr_t:path => $type:ty) => {
        impl From<Type> for $type {
            fn from(arg: Type) -> Self {
                match arg {
                    $expr_t(t) => t,
                    _ => panic!("Recieved unrecognized type"),
                }
            }
        }

        impl From<$type> for Type {
            fn from(val: $type) -> Self {
                $expr_t(val)
            }
        }
    };
}

bijection!(Type::Number => f64);
bijection!(Type::String => String);
bijection!(Type::Function => Function);
bijection!(Type::TimeStep => TimeStep);

impl From<Type> for (f64, f64) {
    fn from(val: Type) -> Self {
        match val {
            Type::TimeStep(t) => (t.price, t.time),
            _ => panic!("Recieved unrecognized type"),
        }
    }
}

impl From<(f64, f64)> for Type {
    fn from(val: (f64, f64)) -> Self {
        Type::TimeStep(TimeStep {
            price: val.0,
            time: val.1,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct TimeStep {
    price: f64,
    time: f64,
}

/// Represents a general runtime concept of a function.
/// It is generic over possible types `Type`.
/// Note that each function must be pure. Meaning it could
/// not contain any side effects. This is guranteed by the
/// `Callable` trait contract whitch takes only immutable
/// reference to self.
pub struct Function(Box<dyn Callable>);

impl Callable for Function {
    fn call(&self, args: &[Type]) -> Type {
        self.0.call(args)
    }
}

/// This is a public Callable trait which lets
/// any function be runable inside.
pub trait Callable {
    fn call(&self, args: &[Type]) -> Type;
}
