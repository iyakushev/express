use super::TimeSeries;
use express::prelude::*;

// FIXME(iy): Doc formula
/**
# Time Weighted Average (TWA)
Computes Time weighted average over arbitrary slice of md
: ts_buffer -- represent each corresponding axes of market data.
: lookback -- represents a window period over time. If the lookback is too large, ma returns None.

# Formula.
![MA formula](https://wikimedia.org/api/rest_v1/media/math/render/svg/a608544726b8de1c3de562245ff0d1cd3d0efad6)
 */
#[allow(dead_code)]
#[inline]
#[runtime_callable(pure)]
pub fn twa(ts_buffer: TimeSeries, lookback: f64) -> Option<f64> {
    let last_tick = ts_buffer.last()?;
    // NOTE(iy): Should this case be cumultive in behavor? E.g. CMA
    if lookback > (last_tick.time - ts_buffer.first()?.time) {
        return None;
    }

    let mut it_buffer = ts_buffer.iter().rev().skip(1).peekable();
    let mut total_time_diff = 0.0;
    let mut prev_tick = last_tick;
    let const_time_diff = last_tick.time - it_buffer.peek()?.time;
    for tick in it_buffer {
        // NOTE(iy): incrementing tick before the check makes it so
        // this tick gets included. Meaning its inclusive on the left
        total_time_diff += prev_tick.time - tick.time;
        if last_tick.time - tick.time > lookback {
            break;
        }
        prev_tick = tick;
    }
    Some((last_tick.price * const_time_diff) / total_time_diff as f64)
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;
    use crate::fill_ts;
    use crate::timeseries::TimeStep;
    use float_cmp::assert_approx_eq;

    #[test]
    pub fn test_smaller_slices() {
        let stack = fill_ts![1.0; 2.0];
        let window = 15.0;
        assert_eq!(twa(Arc::new(stack), window), None)
    }

    #[test]
    pub fn test_full_vec_pass() {
        let stack = fill_ts![1.0, 0.0; 2.0, 1.0; 3.0, 3.0];
        let window = 3.0;
        assert_eq!(twa(Arc::new(stack), window), Some(2.0))
    }

    #[test]
    pub fn test_range_inclusivity() {
        // NOTE(iy): 3 seconds must be passed for computation
        let stack = fill_ts![10.0, 0.0; 11.0, 0.9; 12.0, 3.0; 13.0, 4.0];
        assert_eq!(Arc::new(stack).first().unwrap().price, 10.0);
        let window = 3.0;
        let result = twa(Arc::new(stack), window).unwrap();
        assert_approx_eq!(f64, result, 4.19354838, epsilon = 0.01)
    }
}
