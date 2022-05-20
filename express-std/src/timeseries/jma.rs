use express::types::{Callable, Type};
use express::xmacro::runtime_callable;

use super::TimeSeries;

macro_rules! ezinit {
    ($($name: ident),*; $val: expr) => {
        $(let mut $name = $val;)*
    };
}

#[inline(always)]
fn get_pahse_ratio(phase: f64) -> f64 {
    if phase < -100.0 {
        0.5
    } else if phase > 100.0 {
        2.5
    } else {
        1.5 + phase * 0.01
    }
}

#[inline(always)]
fn get_price_volatility(uband: f64, lband: f64, price: f64) -> f64 {
    let d1 = price - uband;
    let d2 = price - lband;
    if d1 != d2 {
        d1.abs().max(d2.abs())
    } else {
        0.0
    }
}

const JMA_SUMLEN: usize = 10;
const JMA_BANDPERIOD: usize = 65;

/// Jurik Moving Average (JMA)
#[runtime_callable(pure)]
fn jma(ts: TimeSeries, len: usize, phase: f64) -> Option<f64> {
    // setup jma variables
    // Boooooooy it is gonna be slow to compute...
    if ts.is_empty() {
        return None;
    }
    ezinit!(kv, det0, det1, ma2; 0.0);
    ezinit!(upper_band, lower_band, ma1, jma; ts[0].price);
    let length = 0.5 * (len - 1) as f64;
    let length1 = ((length.sqrt().ln() / 2.0f64.ln()) + 2.0).max(0.0);
    let length2 = length1 * length.sqrt();
    let pow1 = 0.5f64.max(length1 - 2.0);
    let bet = length2 / (length2 + 1.0);
    let beta = 0.45 * (len - 1) as f64 / (0.45 * (len - 1) as f64 + 2.0);
    let phase_ratio = get_pahse_ratio(phase);
    ezinit!(volty, vsum; Vec::with_capacity(ts.len()) as Vec<f64>);

    for (idx, tick) in ts.iter().skip(1).enumerate() {
        let price = tick.price;
        // price volatility
        let del1 = price - upper_band;
        let del2 = price - lower_band;
        volty.push(if del1 != del2 {
            del1.abs().max(del2.abs())
        } else {
            0.0
        });

        // relative price volatility factor
        vsum.push(
            vsum[idx - 1] + (volty[idx] - volty[(idx - JMA_SUMLEN).max(0)]) / JMA_SUMLEN as f64,
        );
        let avgvol_slice = &vsum[(idx - JMA_BANDPERIOD).max(0)..=idx];
        let avg_volty: f64 = avgvol_slice.iter().sum::<f64>() / avgvol_slice.len() as f64;
        let d_volty = if avg_volty == 0.0 {
            0.0
        } else {
            volty[idx] / avg_volty
        };
        let r_volty = 1.0f64.max(length1.powf(1.0 / pow1).min(d_volty));

        // Update Jurik volatility bands
        let pow2 = r_volty.powf(pow1);
        kv = bet.powf(pow2.sqrt());
        upper_band = if del1 > 0.0 {
            price
        } else {
            price - (kv * del1)
        };
        lower_band = if del2 > 0.0 {
            price
        } else {
            price - (kv * del2)
        };

        // Jurik dynamic factor
        let power = r_volty.powf(pow1);
        let alpha = beta.powf(power);

        // 1st stage. EMA
        ma1 = ((1.0 - alpha) * price) + (alpha * ma1);

        // 2nd stage. Kalman
        det0 = ((price - ma1) * (1.0 - beta)) + (beta * det0);
        ma2 = ma1 + phase_ratio * det0;

        // 3rd stage - final smoothing by unique Jurik adaptive filter
        det1 = ((ma2 - jma) * (1.0 - alpha).powf(2.0)) + (alpha.powf(2.0) * det1);
        jma = jma + det1;
    }
    Some(jma)
}

#[cfg(test)]
mod tests {
    use express::types::TimeStep;
    use std::sync::Arc;

    use super::*;

    #[test]
    pub fn test_smaller_slices() {
        let stack = fill_ts![2.0; 3.0];
        assert_eq!(jma(Arc::new(stack), 7, 0.0), None)
    }
}
