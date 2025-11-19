use crate::filters::{SpatialFilter, BinLayout};

const C1: f32 = 20.6_f32 * 20.6_f32;
const C2: f32 = 107.7_f32 * 107.7_f32;
const C3: f32 = 737.9_f32 * 737.9_f32;
const C4: f32 = 12194.0_f32 * 12194.0_f32;
const A0: f32 = 1.2589254; // 10^(2/20)

/// A-weighting filter implementation.
/// https://en.wikipedia.org/wiki/A-weighting#A
/// 
/// Attributes:
/// 
/// * weights: Precomputed weights for each frequency bin.
pub struct AWeightingFilter {
    weights: Vec<f32>,
}

impl AWeightingFilter {
    pub fn new() -> Self {
        Self { weights: Vec::new() }
    }

    #[inline]
    fn compute_weight(f: f32) -> f32 {
        if f <= 0.0 { return 0.0; }
        let f2 = f * f;
        let num = C4 * f2 * f2;
        let den = (f2 + C1) * (f2 + C4) * ((f2 + C2) * (f2 + C3)).sqrt();
        let ra = num / den;
        ra * A0
    }
}

impl SpatialFilter for AWeightingFilter {
    fn on_layout_change(&mut self, layout: &BinLayout) {
        self.weights = layout.centers.iter()
            .map(|&f| Self::compute_weight(f))
            .collect();
    }

    fn process(&self, samples: &mut [f32]) {
        for (i, sample) in samples.iter_mut().enumerate() {
            *sample *= self.weights[i];
        }
    }

    fn priority(&self) -> usize { 10 }
}