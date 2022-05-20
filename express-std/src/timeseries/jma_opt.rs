use express::types::{Callable, Type};

struct JMA {
    pub len: f64,
    pub band_period: f64,
    pub sumlen: f64,
    del1: f64,
    del2: f64,
    length: f64,
    length1: f64,
    length2: f64,
    pow1: f64,
    bet: f64,
    beta: f64,
    phase_ratio: f64,
    volty: Vec<f64>,
    vsum: Vec<f64>,
}

impl Callable for JMA {
    fn init(&mut self, args: &[Type]) {}
    fn call(&self, args: &[Type]) -> Option<Type> {
        todo!()
    }

    fn argcnt(&self) -> usize {
        todo!()
    }
}
