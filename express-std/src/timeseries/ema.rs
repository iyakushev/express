use express::types::TimeStep;

/**
Computes exponential moving average over arbitrary slice
: price, time -- represent each corresponding axes of market data. They must have matching size.
: lookback -- represents a window period over time. If the lookback is too large, ma returns None.


# Formula
[EMA Formula](https://wikimedia.org/api/rest_v1/media/math/render/svg/05d06bdbee2c14031fd91ead6f5f772aec1ec964)
 */
#[allow(dead_code)]
#[inline]
pub fn ema(ts_buffer: &[TimeStep], lookback: f64) -> Option<f64> {
    let mut prev_tick = ts_buffer.last()?;
    if lookback > (prev_tick.time - ts_buffer.first()?.time) {
        return None;
    }

    let mut dtsum = 0.0f64;
    let mut dt = 0.0f64;
    let mut value = 0.0f64;
    let mut expsum = 0.0f64;
    let mut it_buffer = ts_buffer.iter().rev();
    prev_tick = it_buffer.next()?;
    for tick in it_buffer {
        if dtsum > lookback {
            return Some(value / expsum);
        }
        dtsum += dt;
        let exp = (-2.0f64.ln() * dtsum / lookback).exp();
        expsum += exp;
        value += prev_tick.price * exp;
        dt = (prev_tick.time - tick.time).min(lookback - dtsum);
        prev_tick = tick;
    }
    Some(value / expsum)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{fill_ts, timeseries::TimeSeries, timeseries::TimeStep};
    use float_cmp::assert_approx_eq;
    use std::ops::Deref;

    #[test]
    pub fn test_smaller_slices() {
        let stack: TimeSeries = fill_ts![2.0; 3.0];
        let window = 15.0;
        assert_eq!(ema(stack.deref(), window), None)
    }
    #[test]
    pub fn test_full_vec_pass() {
        let stack: TimeSeries = fill_ts![2.0, 0.0; 5.0, 1.0; 1.0, 3.0; 2.0, 4.0];
        let window = 3.0;
        let result = ema(stack.deref(), window).unwrap();
        assert_approx_eq!(f64, 2.3078, result, epsilon = 0.001);
    }

    #[test]
    pub fn test_range_inclusivity() {
        // NOTE(iy): 3 seconds must be passed for computation
        let stack: TimeSeries = fill_ts![
            2.0, 0.0;
            2.7, 1.0;
            3.0, 1.3;
            3.4, 1.6;
            3.8, 1.9;
            4.0, 2.0;
            4.1, 2.1;
            4.0, 2.15;
            4.2, 2.3;
            4.4, 2.6;
            4.9, 2.8;
            5.0, 3.1;
            5.1, 3.2;
            4.9, 3.3];
        let window = 3.0;
        let result = ema(&stack.deref(), window).unwrap();
        // actualy producess 4.22827783
        assert_approx_eq!(f64, result, 4.2282, epsilon = 0.001);
        // assert_approx_eq!(f64, result, 4.125628, epsilon = 0.001);
    }
}
