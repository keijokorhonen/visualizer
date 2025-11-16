pub struct GaussianFilter {
    pub sigma: f32,
    pub radius: usize,
    pub kernel: Vec<f32>,
}

impl GaussianFilter {
    pub fn new(sigma: f32, radius: usize) -> Self {
        Self {
            sigma,
            radius,
            kernel: Self::compute_kernel(sigma, radius),
        }
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
}

impl crate::filters::SpatialFilter for GaussianFilter {
    /// Apply Gaussian filter to the input samples in-place.
    /// Convolves the samples with the Gaussian kernel.
    fn process(&self, samples: &mut [f32]) {
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
}