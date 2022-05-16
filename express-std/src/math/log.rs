#![allow(dead_code)]
extern crate express;
use express::types::{Callable, Type};
use express::xmacro::runtime_callable;

/// Clalculates logarithm of a __value__ with by a given __base__
#[runtime_callable]
pub fn log(base: f64, value: f64) -> Option<f64> {
    Some(value.log(base))
}
