#![allow(dead_code)]
use express::prelude::*;

/// Clalculates logarithm of a __value__ with by a given __base__
#[runtime_callable(pure)]
pub fn log(base: f64, value: f64) -> Option<f64> {
    Some(value.log(base))
}

/// Clalculates natural logarithm of a __value__
#[runtime_callable(pure)]
pub fn ln(value: f64) -> Option<f64> {
    Some(value.ln())
}
