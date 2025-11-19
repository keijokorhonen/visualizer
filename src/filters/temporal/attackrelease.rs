use crate::filters::TemporalFilter;

/// Attack-release filter.
/// Uses different smoothing factors for attack and release phases.
/// Applies: attack_alpha * y[n] + (1 - attack_alpha) * x[n]  if x[n] > y[n]
///        release_alpha * y[n] + (1 - release_alpha) * x[n]  if x[n] <= y[n]
/// where y is the previous output and x is the new input.
pub struct AttackReleaseFilter {
    pub attack_alpha: f32,
    pub release_alpha: f32,
    pub prev: Vec<f32>,
}

impl AttackReleaseFilter {
    pub fn new(attack_alpha: f32, release_alpha: f32) -> Self {
        Self {
            attack_alpha,
            release_alpha,
            prev: Vec::new(),
        }
    }
}

impl TemporalFilter for AttackReleaseFilter {
    fn process(&mut self, samples: &mut [f32]) {
        if self.prev.len() != samples.len() {
            self.prev.resize(samples.len(), 0.0);
        }
        for (i, x) in samples.iter_mut().enumerate() {
            let alpha = if *x > self.prev[i] {
                self.attack_alpha
            } else {
                self.release_alpha
            };
            let y = alpha * self.prev[i] + (1.0 - alpha) * *x;
            self.prev[i] = y;
            *x = y;
        }
    }
}