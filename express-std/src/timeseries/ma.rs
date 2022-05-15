use super::TimeStep;
use express::xmacro::runtime_callable;
use std::sync::Arc;

/**
# Simple Moving Average (SMA/MA) trait
Computes simple moving average over arbitrary slice of md
: price, time -- represent each corresponding axes of market data. They must have matching size.
: lookback -- represents a window period over time. If the lookback is too large, ma returns None.

# Formula
![MA formula](https://wikimedia.org/api/rest_v1/media/math/render/svg/a608544726b8de1c3de562245ff0d1cd3d0efad6)
 */
#[allow(dead_code)]
#[runtime_callable]
pub fn ma(ts_buffer: Arc<[TimeStep]>, lookback: f64) -> Option<f64> {
    let last_tick = ts_buffer.last()?.time;
    // NOTE(iy): Should this case be cumultive in behavor? E.g. CMA
    if lookback > (last_tick - ts_buffer.first()?.time) {
        return None;
    }
    let mut sum = 0.0;
    let mut total_len = ts_buffer.len();
    for (pos, tick) in ts_buffer.iter().enumerate().rev() {
        // NOTE(iy): this makes ma range inclusive on the left
        // meaning that given time = [0, 1, 2, 3, 4] and lookback = 3
        // it would actually compute the whole slice,
        // since 4 - 1 = 3 exactly we need 1 extra step to satisfy condition
        sum += tick.price;
        if last_tick - tick.time > lookback {
            total_len -= pos;
            break;
        }
    }
    Some(sum / total_len as f64)
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use super::ma;
    use crate::fill_ts;
    use crate::timeseries::{TimeSeries, TimeStep};

    #[test]
    pub fn test_smaller_slices() {
        let stack: TimeSeries = fill_ts![1.0; 2.0];
        let window = 15.0;
        assert_eq!(ma(stack.deref().into(), window), None)
    }

    #[test]
    pub fn test_full_vec_pass() {
        let stack: TimeSeries = fill_ts![1.0, 0.0; 2.0, 1.0; 3.0, 3.0];
        let window = 3.0;
        assert_eq!(ma(stack.deref().into(), window), Some(2.0))
    }

    #[test]
    pub fn test_range_inclusivity() {
        // NOTE(iy): 3 seconds must be passed for computation
        let stack: TimeSeries = fill_ts![10.0, 0.0; 11.0, 0.9; 12.0, 3.0; 13.0, 4.0];
        assert_eq!(stack.deref().first().unwrap().price, 10.0);
        let window = 3.0;
        assert_eq!(ma(stack.deref().into(), window), Some(12.0))
    }
}
