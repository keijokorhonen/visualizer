pub mod attackrelease;
pub mod exponential;
pub mod peakholddecay;

pub use attackrelease::AttackReleaseFilter;
pub use exponential::ExponentialFilter;
pub use peakholddecay::PeakHoldDecayFilter;

use crate::frontend::egui_frontend::UiComponent;

pub trait TemporalFilter: Send + Sync + UiComponent {
    fn process(&mut self, samples: &mut [f32]);
    fn state_vec(&mut self) -> Option<&mut Vec<f32>> {
        None
    }

    fn reset(&mut self) {
        if let Some(v) = self.state_vec() {
            v.fill(0.0);
        }
    }
}
