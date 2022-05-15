#![allow(dead_code)]
extern crate xmacro;
use xmacro::{resolve_name, runtime_callable};

mod express {
    pub use types;
}

#[runtime_callable]
fn foo(input: f64) -> Option<f64> {
    Some(input * 2.0 + 2.0)
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
        let foo = resolve_name!(upper);
    }
}
