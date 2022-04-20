/**
# Simple Moving Average (SMA/MA)
Computes simple moving average over arbitrary slice of md
: price, time -- represent each corresponding axes of market data. They must have matching size.
: lookback -- represents a window period over time. If the lookback is too large, ma returns None.

# Formula
![MA formula](https://wikimedia.org/api/rest_v1/media/math/render/svg/a608544726b8de1c3de562245ff0d1cd3d0efad6)
*/
#[inline]
pub fn ma(price: &[f64], time: &[f64], lookback: f64) -> Option<f64> {
    //TODO(iy): price and time_ns should probably be warped into TimeSeries struct
    let last_tick = time.last()?;
    // NOTE(iy): Should this case be cumultive in behavor? E.g. CMA
    if lookback > (last_tick - time.first()?) || price.len() != time.len() {
        return None;
    }
    let mut sum = 0.0;

    for (pos, pr) in price.iter().enumerate().rev() {
        if last_tick - time[pos] > lookback {
            // NOTE(iy): this makes ma range inclusive on the left
            // meaning that given time = [0, 1, 2, 3, 4] and lookback = 3
            // it would actually compute the whole slice,
            // since 4 - 1 = 3 exactly we need 1 extra step to satisfy condition
            sum += pr;
            return Some(sum / (time.len() - pos) as f64);
        }
        sum += pr;
    }
    Some(sum / price.len() as f64)
}

#[cfg(test)]
mod test {
    use super::ma;
    //NOTE(iy): implementation detail.
    // In the real world price/time_s
    // must use something of a VecDeque
    // or optimally other implementations
    // of a RingBuffer.

    #[test]
    pub fn test_smaller_slices() {
        let price = vec![1.0, 2.0];
        let time_s = vec![1.0, 2.0];
        let window = 15.0;
        assert_eq!(ma(&price, &time_s, window), None)
    }

    #[test]
    pub fn test_mismatched_vecs() {
        let price = vec![1.0, 2.0];
        let time_s = vec![1.0, 2.0, 3.0];
        let window = 1.0;
        assert_eq!(ma(&price, &time_s, window), None)
    }

    #[test]
    pub fn test_full_vec_pass() {
        let price = vec![1.0, 2.0, 3.0];
        let time_s = vec![0.0, 1.0, 3.0];
        let window = 3.0;
        assert_eq!(ma(&price, &time_s, window), Some(2.0))
    }

    #[test]
    pub fn test_range_inclusivity() {
        // NOTE(iy): 3 seconds must be passed for computation
        let price = vec![10.0, 11.0, 12.0, 13.0];
        let time_s = vec![0.0, 0.9, 3.0, 4.0];
        let window = 3.0;
        assert_eq!(ma(&price, &time_s, window), Some(12.0))
    }
}
