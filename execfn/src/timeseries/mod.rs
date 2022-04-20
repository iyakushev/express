/// Simplifies module registration
/// Each timeseries filename and fn
/// mast match exactly.
/// Example:
/// ```rust
/// // timeseries::ma.rs
/// pub fn ma(...) -> f64{ ... }
/// ```
macro_rules! import {
    ($name:ident) => {
        mod $name;
        pub use $name::$name;
    };
}

import!(ema);
import!(ma);
