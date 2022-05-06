extern crate express;
use express::types;
use express::xmacro::runtime_callable;

/// Clalculates logarithm of a __value__ with by a given __base__
#[runtime_callable]
fn log(base: f64, value: f64) -> f64 {
    value.log(base)
}
