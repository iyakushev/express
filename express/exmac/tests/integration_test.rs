use exmac::runtime_callable;
use types::{Callable, Type};

#[runtime_callable]
fn foo(input: f64) -> f64 {
    input * 2.0 + 2.0
}
