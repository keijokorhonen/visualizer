pub mod exponential;
pub mod attackrelease;
pub mod peakholddecay;

pub use exponential::ExponentialFilter;
pub use attackrelease::AttackReleaseFilter;
pub use peakholddecay::PeakHoldDecayFilter;

pub trait TemporalFilter: Send + Sync {
    fn process(&mut self, samples: &mut [f32]);
    fn state_vec(&mut self) -> Option<&mut Vec<f32>> { None }

    fn reset(&mut self) {
        if let Some(v) = self.state_vec() {
            v.fill(0.0);
        }
    }
}