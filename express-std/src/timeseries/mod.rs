pub use express::types::TimeStep;
use std::sync::Arc;

/**
It accumulates ticks over time into the inner ring buffer
and computes specified functions on it.
*/
#[allow(dead_code)]
pub type TimeSeries = Arc<[TimeStep]>;

/**
Fills time series with values just like `vec![...]`.
: `fill_ts![1.0; 2.0; ...]` -- would push price and time as the same;
: `fill_ts![1.0, 0.0; 2.0, 1.0;]` -- would split by `,` and push first arg as price, second would be time.
 */
#[cfg(test)]
#[macro_export]
macro_rules! fill_ts {

    [$($val:expr); +$(,)?] => {{
        [$(TimeStep{ price: $val, time: $val },)*]
    }};
    [$($val:expr, $t:expr);+$(,)?] => {{
        [$(TimeStep{ price: $val, time: $t }),*]
    }}
}

mod ema;
mod jma;
mod ma;
mod malin;
mod twa;

pub use ema::*;
pub use jma::*;
pub use ma::*;
pub use malin::*;
pub use twa::*;
