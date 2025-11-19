pub mod gaussian;
pub mod eq_curve;
pub mod a_weighting;

pub use gaussian::GaussianFilter;
pub use eq_curve::EqCurveFilter;
pub use a_weighting::AWeightingFilter;


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

    fn priority(&self) -> usize { 50 }
}