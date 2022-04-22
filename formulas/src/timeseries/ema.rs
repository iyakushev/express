use super::TimeStep;

/**
Computes exponential moving average over arbitrary slice
: price, time -- represent each corresponding axes of market data. They must have matching size.
: lookback -- represents a window period over time. If the lookback is too large, ma returns None.


# Formula
[EMA Formula](https://wikimedia.org/api/rest_v1/media/math/render/svg/05d06bdbee2c14031fd91ead6f5f772aec1ec964)
*/
pub fn ema(ts_buffer: &[TimeStep], lookback: f64) -> Option<f64> {
    let last_tick = ts_buffer.last()?.time;
    // NOTE(iy): Should this case be cumultive in behavor? E.g. CMA
    if lookback > (last_tick - ts_buffer.first()?.time) {
        return None;
    }

    let mut cumdt = 0.0f64;
    let mut value = 0.0f64;
    let mut expsum = 0.0f64;
    for (pos, p) in ts_buffer.iter().skip(1).rev().enumerate() {
        if cumdt < lookback {
            return Some(value / expsum);
        }
    }
    None
}

// #[cfg(test)]
// mod test {
//     use super::ema;
//     #[test]
//     pub fn test_smaller_slices() {
//         let price = vec![1.0, 2.0];
//         let time_s = vec![1.0, 2.0];
//         let window = 15.0;
//         assert_eq!(ema(&price, &time_s, window), None)
//     }
//
//     #[test]
//     pub fn test_mismatched_vecs() {
//         let price = vec![1.0, 2.0];
//         let time_s = vec![1.0, 2.0, 3.0];
//         let window = 1.0;
//         assert_eq!(ema(&price, &time_s, window), None)
//     }
//
//     #[test]
//     pub fn test_full_vec_pass() {
//         let price = vec![1.0, 2.0, 3.0];
//         let time_s = vec![0.0, 1.0, 3.0];
//         let window = 3.0;
//         assert_eq!(ema(&price, &time_s, window), Some(2.0))
//     }
//
//     #[test]
//     pub fn test_range_inclusivity() {
//         // NOTE(iy): 3 seconds must be passed for computation
//         let price = vec![2.0, 5.0, 1.0, 6.25];
//         let time_s = vec![0.0, 0.9, 3.0, 4.0];
//         let window = 3.0;
//         assert_eq!(ema(&price, &time_s, window), Some(4.25))
//     }
// }
