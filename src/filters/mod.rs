pub mod gaussian;
pub use gaussian::GaussianFilter;

pub mod exponential;
pub use exponential::ExponentialFilter;

pub trait SpatialFilter: Send + Sync {
    fn process(&self, samples: &mut [f32]);
}

pub trait TemporalFilter: Send + Sync {
    fn process(&mut self, samples: &mut [f32]);

    fn reset(&mut self);
}