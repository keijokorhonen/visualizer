use crate::filters::TemporalFilter;

/// Peak-hold-decay filter.
/// Holds the peak value and decays it exponentially over time.
/// Applies: y[n] = max(x[n], decay * y[n-1])
/// where y is the previous output, x is the new input, and decay is the decay factor.
pub struct PeakHoldDecayFilter {
    pub decay: f32,
    pub prev: Vec<f32>,
}

impl PeakHoldDecayFilter {
    pub fn new(decay: f32) -> Self {
        Self {
            decay,
            prev: Vec::new(),
        }
    }
}

impl TemporalFilter for PeakHoldDecayFilter {
    fn process(&mut self, samples: &mut [f32]) {
        if self.prev.len() != samples.len() {
            self.prev.resize(samples.len(), 0.0);
        }
        for (i, x) in samples.iter_mut().enumerate() {
            let y = x.max(self.decay * self.prev[i]);
            self.prev[i] = y;
            *x = y;
        }
    }
}