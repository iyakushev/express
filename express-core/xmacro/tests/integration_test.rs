#![allow(dead_code)]
extern crate xmacro;
use types::{Callable, CallableType, Type};
use xmacro::{resolve_name, runtime_callable};

mod express {
    pub use types;
}

#[runtime_callable(pure)]
fn foo(input: f64) -> f64 {
    input * 2.0 + 2.0
}

#[runtime_callable(const)]
fn bar(input: f64) -> f64 {
    input * 2.0 + 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expansion() {
        #[runtime_callable]
        fn upper(input: String) -> Option<String> {
            Some(input.to_uppercase())
        }
        let _ = resolve_name!(upper);
    }
}
