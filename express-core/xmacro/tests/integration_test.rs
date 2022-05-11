extern crate xmacro;
use xmacro::runtime_callable;

#[runtime_callable]
fn foo(input: f64) -> f64 {
    input * 2.0 + 2.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expansion() {
        #[runtime_callable]
        fn upper(input: String) -> String {
            input.to_uppercase()
        }
    }
}
