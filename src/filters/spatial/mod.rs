pub mod a_weighting;
pub mod eq_curve;
pub mod gaussian;

pub use a_weighting::AWeightingFilter;
pub use eq_curve::EqCurveFilter;
pub use gaussian::GaussianFilter;

use crate::frontend::egui_frontend::UiComponent;

// Bin layout info passed to filters needing bin center frequencies.
#[derive(Clone)]
pub struct BinLayout {
    pub centers: Vec<f32>,
    pub min_freq: f32,
    pub max_freq: f32,
    pub log_min: f32,
    pub log_max: f32,
    pub spacing_log: bool,
}

impl BinLayout {
    pub fn build_layout(num_bins: usize, min_freq: f32, max_freq: f32, log: bool) -> BinLayout {
        let min_freq = min_freq.max(1e-6);
        let max_freq = max_freq.max(min_freq + 1.0);
        let log_min = min_freq.ln();
        let log_max = max_freq.ln();
        let mut centers = Vec::with_capacity(num_bins);
        for i in 0..num_bins {
            let t = (i as f32 + 0.5) / num_bins as f32;
            let f = if log {
                (log_min + t * (log_max - log_min)).exp()
            } else {
                min_freq + t * (max_freq - min_freq)
            };
            centers.push(f);
        }
        BinLayout {
            centers,
            min_freq,
            max_freq,
            log_min,
            log_max,
            spacing_log: log,
        }
    }
}

pub trait SpatialFilter: Send + Sync + UiComponent {
    fn on_layout_change(&mut self, _layout: &BinLayout) {}

    fn process(&self, samples: &mut [f32]);

    fn priority(&self) -> usize {
        50
    }
}
