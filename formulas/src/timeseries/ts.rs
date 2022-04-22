use slice_deque::SliceDeque;
use std::ops::Deref;
#[allow(unused, dead_code)]
#[derive(Debug)]
pub struct TimeStep {
    pub price: f64,
    pub time: f64,
}

pub struct TimeSeriesFn {
    inner_window: SliceDeque<TimeStep>, // NOTE(iy): This might be other RingBuffer if perf is bad
}

/**
Windowed struct that implements basic time series traits.
It accumulates ticks over time into the inner ring buffer
and computes specified functions on it.
*/
impl TimeSeriesFn {
    pub fn new(cap: usize) -> Self {
        TimeSeriesFn {
            inner_window: SliceDeque::with_capacity(cap),
        }
    }

    #[inline]
    pub fn slice(&self) -> &[TimeStep] {
        self.inner_window.deref()
    }

    #[inline]
    pub fn push(&mut self, ts: TimeStep) {
        // This might be a concern for perf
        // but for now it is infinitely growable
        self.inner_window.push_back(ts);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner_window.clear();
    }
}
