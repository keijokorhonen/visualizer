pub mod a_weighting;
pub mod eq_curve;
pub mod gaussian;

pub use a_weighting::AWeightingFilter;
pub use eq_curve::EqCurveFilter;
pub use gaussian::GaussianFilter;

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

pub trait SpatialFilter: Send + Sync {
    fn on_layout_change(&mut self, _layout: &BinLayout) {}

    fn process(&self, samples: &mut [f32]);

    fn priority(&self) -> usize {
        50
    }
}
