use std::{
    fmt::{Debug, Display},
    rc::Rc,
    sync::Arc,
};

/// Representation of valid runtime types.
/// Every function that implements [Callable] trait must
/// accept and return on of the following types. End user
/// doesn't need to care about them thanks to the `#[runtime_callable]`
/// macro which expands function declaration into a callable ZST structure
/// with its arguments and return type being automatically converted via `From` trait implementations.
#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Number(f64),
    String(String),
    Collection(Arc<[TimeStep]>),
    TimeStep(TimeStep),
    // Function(Function),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Number(num) => write!(f, "{}", num),
            Type::String(string) => write!(f, "{}", string),
            Type::Collection(coll) => write!(f, "{:?}", *coll),
            Type::TimeStep(ts) => write!(f, "{}", ts),
        }
    }
}

/// A wrapping structure around `(f64, f64)` that represents
/// a single tick of data with fields.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct TimeStep {
    pub price: f64,
    pub time: f64,
}

impl Display for TimeStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TimeStep<price: {}, time: {}>", self.price, self.time)
    }
}

/// Represents a general runtime concept of a function.
/// It is generic over possible types `Type`.
/// Note that each function must be pure. Meaning it could
/// not contain any side effects. This is guranteed by the
/// `Callable` trait contract whitch takes only immutable
/// reference to self.
pub type Function = Rc<dyn Callable>;

// impl Debug for Function {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Function")
//     }
// }

/// This is a public Callable trait which lets
/// any function be runable inside.
/// ## Why?
/// Because rust calling conventions (Fn traits) are still unstable.
/// Here is a [tracking issue](https://doc.rust-lang.org/stable/std/ops/trait.Fn.html#required-methods)
/// ## Safety
/// Calling `call` method directly on #[runtime_callable] objects is unsafe
/// since they access arguments with `get_unchecked(pos)`.
/// The arg count check is performed during AST creation.
pub trait Callable {
    /// Execuded once before the main loop with call
    /// Allows struct to initialize its internal state.
    fn init(&mut self, args: &[Type]);

    // One day we will get Trait const fn
    /// Returns the name of an object.
    fn name(&self) -> &str;

    // fn init(args: &[Type]) -> Self;
    fn call(&self, args: &[Type]) -> Option<Type>;

    /// Returns a number of arguments the function expects
    fn argcnt(&self) -> usize;

    /// Signifies if the Callable object stands for a pure function.
    /// If all of its arguments are Const as well (or pure functions with const args).
    /// Then the call can be performed at the graph "build time" rather than runtime.
    fn is_pure(&self) -> bool {
        false
    }
}

/// Automatically implements bijection conversion traits
/// for types __Type(T) <-> T__.
/// Note that `T` and `Type` are owned values.
#[macro_export]
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
// bijection!(Type::Function => Function);
bijection!(Type::TimeStep => TimeStep);
bijection!(Type::Collection => Arc<[TimeStep]>);

impl From<&Type> for f64 {
    fn from(val: &Type) -> Self {
        match val {
            Type::Number(n) => *n,
            _ => panic!("Recieved unrecognized type"),
        }
    }
}

impl From<&Type> for TimeStep {
    fn from(val: &Type) -> Self {
        match val {
            Type::TimeStep(ts) => *ts,
            _ => panic!("Recieved unrecognized type"),
        }
    }
}

impl From<&Type> for String {
    fn from(val: &Type) -> Self {
        match val {
            Type::String(n) => n.clone(),
            _ => panic!("Recieved unrecognized type"),
        }
    }
}

impl From<&Type> for Arc<[TimeStep]> {
    fn from(val: &Type) -> Self {
        match val {
            Type::Collection(c) => c.clone(),
            _ => panic!("Recieved unrecognized type"),
        }
    }
}

macro_rules! convert_number {
    ($num_t: tt) => {
        impl From<Type> for $num_t {
            fn from(val: Type) -> Self {
                match val {
                    Type::Number(n) => n as $num_t,
                    _ => panic!("Recieved unrecognized type"),
                }
            }
        }
        impl From<&Type> for $num_t {
            fn from(val: &Type) -> Self {
                match val {
                    Type::Number(n) => *n as $num_t,
                    _ => panic!("Recieved unrecognized type"),
                }
            }
        }

        impl From<$num_t> for Type {
            fn from(val: $num_t) -> Self {
                Type::Number(val as f64)
            }
        }
    };
}

convert_number!(isize);
convert_number!(usize);
convert_number!(i32);
convert_number!(i64);
convert_number!(f32);

// NOTE(iy): Additional conversion for tuple f64 into TimeStep
// might be useful.
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_bijection {
        ($val:expr, $src_t:ty) => {
            let v = $val;
            let t = Type::from(v);
            let v = <$src_t>::from(t);
            let _: Type = v.into();
        };
    }

    #[test]
    fn test_f64() {
        test_bijection!(0.12f64, f64);
    }

    #[test]
    fn test_string() {
        test_bijection!("abcd".to_string(), String);
    }

    #[test]
    fn test_tuple() {
        test_bijection!((0.1, 2.2), (f64, f64));
    }

    #[test]
    fn test_timestep() {
        test_bijection!(
            TimeStep {
                price: 22.0,
                time: 11.0
            },
            TimeStep
        );
    }
}
