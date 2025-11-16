use crate::filters::TemporalFilter;

pub struct ExponentialFilter {
    alpha: f32,
    prev: Vec<f32>,
}

impl ExponentialFilter {
    pub fn new(alpha: f32) -> Self {
        Self { alpha, prev: Vec::new() }
    }
}

impl TemporalFilter for ExponentialFilter {
    fn process(&mut self, samples: &mut [f32]) {
        if self.prev.len() != samples.len() {
            self.prev.resize(samples.len(), 0.0);
        }
        for (i, x) in samples.iter_mut().enumerate() {
            let y = self.alpha * *x + (1.0 - self.alpha) * self.prev[i];
            self.prev[i] = y;
            *x = y;
        }
    }
    fn reset(&mut self) {
        self.prev.fill(0.0);
    }
}