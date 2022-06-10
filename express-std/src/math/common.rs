use express::types::{Callable, Type};
use express::xmacro::runtime_callable;

#[runtime_callable(pure)]
fn max(lhs: f64, rhs: f64) -> f64 {
    lhs.max(rhs)
}

#[runtime_callable(pure)]
fn min(lhs: f64, rhs: f64) -> f64 {
    lhs.min(rhs)
}
