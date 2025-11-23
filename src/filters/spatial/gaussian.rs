use crate::filters::SpatialFilter;

/// Gaussian spatial filter.
/// Applies a Gaussian blur to the input samples.
/// The filter is defined by its standard deviation (sigma) and radius.
/// The kernel is computed once during initialization.
/// The filter can be applied multiple times (num_passes) for a stronger effect.
pub struct GaussianFilter {
    pub sigma: f32,
    pub radius: usize,
    pub num_passes: usize,
    pub kernel: Vec<f32>,
    pub enabled: bool,
}

impl GaussianFilter {
    pub fn new(sigma: f32, radius: usize, num_passes: usize) -> Self {
        Self {
            sigma,
            radius,
            num_passes,
            kernel: Self::compute_kernel(sigma, radius),
            enabled: true,
        }
    }

    pub fn default() -> Self {
        let sigma = 2.0;
        let radius = 4;
        let num_passes = 3;
        Self::new(sigma, radius, num_passes)
    }

    fn compute_kernel(sigma: f32, radius: usize) -> Vec<f32> {
        let mut kernel = Vec::with_capacity(2 * radius + 1);
        let denom = 2.0 * sigma * sigma;
        let mut sum = 0.0;

        for x in -(radius as isize)..=(radius as isize) {
            let value = (-(x as f32 * x as f32) / denom).exp();
            kernel.push(value);
            sum += value;
        }

        // Normalize kernel
        for k in kernel.iter_mut() {
            *k /= sum;
        }
        kernel
    }

    fn apply_single_pass(&self, samples: &mut [f32]) {
        let mut out = vec![0.0f32; samples.len()];

        let num_samples = samples.len();

        for i in 0..num_samples {
            let mut acc = 0.0;
            for (k, &weight) in self.kernel.iter().enumerate() {
                let sample_index = i as isize + k as isize - self.radius as isize;
                if sample_index >= 0 && sample_index < num_samples as isize {
                    acc += samples[sample_index as usize] * weight;
                }
            }
            out[i] = acc;
        }

        samples.copy_from_slice(&out);
    }

    pub fn recompute_if_needed(&mut self, old_sigma: f32, old_radius: usize) {
        if (self.sigma - old_sigma).abs() > f32::EPSILON || self.radius != old_radius {
            self.kernel = Self::compute_kernel(self.sigma, self.radius);
        }
    }
}

impl SpatialFilter for GaussianFilter {
    /// Apply Gaussian filter to the input samples in-place.
    /// Convolves the samples with the Gaussian kernel.
    fn process(&self, samples: &mut [f32]) {
        if self.enabled {
            for _ in 0..self.num_passes {
                self.apply_single_pass(samples);
            }
        }
    }
}
