use super::TimeStep;

// FIXME(iy): Doc formula
/**
# Linear Moving Average (MALin)
Computes linear moving average over arbitrary slice of md
: ts_buffer -- represent each corresponding axes of market data.
: lookback -- represents a window period over time. If the lookback is too large, ma returns None.

# Formula.
![MA formula](https://wikimedia.org/api/rest_v1/media/math/render/svg/a608544726b8de1c3de562245ff0d1cd3d0efad6)
 */
#[allow(dead_code)]
#[inline]
pub fn malin(ts_buffer: &[TimeStep], lookback: f64) -> Option<f64> {
    let last_tick = ts_buffer.last()?;
    // NOTE(iy): Should this case be cumultive in behavor? E.g. CMA
    if lookback > (last_tick.time - ts_buffer.first()?.time) {
        return None;
    }

    let mut total_ws = 0;

    for (pos, t) in ts_buffer.iter().rev().enumerate() {
        // NOTE(iy): incrementing tick before the check makes it so
        // this tick gets included. Meaning its inclusive on the left
        total_ws += pos + 1;
        if last_tick.time - t.time > lookback {
            return Some((last_tick.price * (pos + 1) as f64) / total_ws as f64);
        }
    }
    Some((last_tick.price * ts_buffer.len() as f64) / total_ws as f64)
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use super::*;
    use crate::fill_ts;
    use crate::timeseries::{TimeSeries, TimeStep};

    #[test]
    pub fn test_smaller_slices() {
        let stack: TimeSeries = fill_ts![1.0; 2.0];
        let window = 15.0;
        assert_eq!(malin(stack.deref(), window), None)
    }

    #[test]
    pub fn test_full_vec_pass() {
        let stack: TimeSeries = fill_ts![1.0, 0.0; 2.0, 1.0; 3.0, 3.0];
        let window = 3.0;
        assert_eq!(malin(stack.deref(), window), Some(1.5))
    }

    #[test]
    pub fn test_range_inclusivity() {
        // NOTE(iy): 3 seconds must be passed for computation
        let stack: TimeSeries = fill_ts![10.0, 0.0; 11.0, 0.9; 12.0, 3.0; 13.0, 4.0];
        assert_eq!(stack.deref().first().unwrap().price, 10.0);
        let window = 3.0;
        assert_eq!(malin(stack.deref(), window), Some(6.5))
    }
}
