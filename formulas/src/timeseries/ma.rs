use super::TimeSeriesFn;

/**
# Simple Moving Average (SMA/MA) trait
Computes simple moving average over arbitrary slice of md
: price, time -- represent each corresponding axes of market data. They must have matching size.
: lookback -- represents a window period over time. If the lookback is too large, ma returns None.

# Formula
![MA formula](https://wikimedia.org/api/rest_v1/media/math/render/svg/a608544726b8de1c3de562245ff0d1cd3d0efad6)
 */
pub trait MA {
    fn ma(&self, lookback: f64) -> Option<f64>;
}

impl MA for TimeSeriesFn {
    fn ma(&self, lookback: f64) -> Option<f64> {
        let ts_buffer = self.slice();
        let last_tick = ts_buffer.last()?.time;
        // NOTE(iy): Should this case be cumultive in behavor? E.g. CMA
        if lookback > (last_tick - ts_buffer.first()?.time) {
            return None;
        }
        let mut sum = 0.0;

        for (pos, tick) in ts_buffer.iter().enumerate().rev() {
            // NOTE(iy): this makes ma range inclusive on the left
            // meaning that given time = [0, 1, 2, 3, 4] and lookback = 3
            // it would actually compute the whole slice,
            // since 4 - 1 = 3 exactly we need 1 extra step to satisfy condition
            sum += tick.price;
            if last_tick - tick.time > lookback {
                return Some(sum / (ts_buffer.len() - pos) as f64);
            }
        }
        Some(sum / ts_buffer.len() as f64)
    }
}

#[cfg(test)]
mod test {
    use crate::timeseries::{ma::MA, TimeStep};

    use super::TimeSeriesFn;
    //NOTE(iy): implementation detail.
    // In the real world price/time_s
    // must use something of a VecDeque
    // or optimally other implementations
    // of a RingBuffer.

    macro_rules! fill_ts {
	    [$($val:expr); +$(,)?] => {
            {
                let mut stack = TimeSeriesFn::new(10);
                $(stack.push(TimeStep{ price: $val, time: $val });)*
                stack
            }
	    };
        [$($val:expr, $t:expr);+$(,)?] => {
            {
                let mut stack = TimeSeriesFn::new(10);
                $(stack.push(TimeStep{ price: $val, time: $t });)*
                stack
            }
        }
    }

    #[test]
    pub fn test_smaller_slices() {
        let stack = fill_ts![1.0; 2.0];
        let window = 15.0;
        assert_eq!(stack.ma(window), None)
    }

    #[test]
    pub fn test_full_vec_pass() {
        let stack = fill_ts![1.0, 0.0; 2.0, 1.0; 3.0, 3.0];
        let window = 3.0;
        assert_eq!(stack.ma(window), Some(2.0))
    }

    #[test]
    pub fn test_range_inclusivity() {
        // NOTE(iy): 3 seconds must be passed for computation
        let stack = fill_ts![10.0, 0.0; 11.0, 0.9; 12.0, 3.0; 13.0, 4.0];
        assert_eq!(stack.slice().first().unwrap().price, 10.0);
        let window = 3.0;
        assert_eq!(stack.ma(window), Some(12.0))
    }
}
