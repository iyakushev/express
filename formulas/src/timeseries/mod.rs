use once_cell::sync::Lazy;
use slice_deque::SliceDeque;
#[derive(Debug)]
pub struct TimeStep {
    pub price: f64,
    pub time: f64,
}

/**
It accumulates ticks over time into the inner ring buffer
and computes specified functions on it.
*/
#[allow(dead_code)]
pub type TimeSeries = SliceDeque<TimeStep>;

static TIME_SERIES: Lazy<SliceDeque<TimeSeries>> = Lazy::new(|| SliceDeque::with_capacity(100));

/**
Fills time series with values just like `vec![...]`.
: `fill_ts![1.0; 2.0; ...]` -- would push price and time as the same;
: `fill_ts![1.0, 0.0; 2.0, 1.0;]` -- would split by `,` and push first arg as price, second would be time.
 */
#[cfg(test)]
#[macro_export]
macro_rules! fill_ts {
    [$($val:expr); +$(,)?] => {
        {
            let mut stack = TimeSeries::with_capacity(10);
            $(stack.push_back(TimeStep{ price: $val, time: $val });)*
            stack
        }
    };
    [$($val:expr, $t:expr);+$(,)?] => {
        {
            let mut stack = TimeSeries::with_capacity(10);
            $(stack.push_back(TimeStep{ price: $val, time: $t });)*
            stack
        }
    }
}
mod ema;
mod ma;
