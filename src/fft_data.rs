use std::sync::{Arc, Mutex};

use spectrum_analyzer::{
    FrequencySpectrum,
    samples_fft_to_spectrum,
    FrequencyLimit,
};
use spectrum_analyzer::scaling::divide_by_N_sqrt;
use spectrum_analyzer::windows::hann_window;

use crate::filters::{
    SpatialFilter,
    GaussianFilter,
    TemporalFilter,
    ExponentialFilter
};

#[derive(Clone)]
pub struct FFTData {
    pub spectrum: Arc<Mutex<Option<FrequencySpectrum>>>,
    pub sample_rate: u32,
    pub window_size: usize,
    pub min_frequency: f32,
    pub max_frequency: f32,
    pub spatial_filter: Option<Arc<dyn SpatialFilter>>,
    pub temporal_filter: Option<Arc<Mutex<dyn TemporalFilter>>>,
}

impl FFTData {
    /// Default constructor. Frequency limits set to 20 Hz - Nyquist.
    pub fn new(sample_rate: u32, window_size: usize) -> Self {
        FFTData {
            spectrum: Arc::new(Mutex::new(None)),
            sample_rate,
            window_size,
            min_frequency: 20.0,
            max_frequency: sample_rate as f32 / 2.0,
            spatial_filter: Some(Arc::new(GaussianFilter::new(5.0, 3))),
            temporal_filter: Some(Arc::new(Mutex::new(ExponentialFilter::new(0.2))))
        }
    }

    pub fn update_spectrum(&self, samples: &[f32]) {
        let window = hann_window(&samples[0..self.window_size]);
        let spectrum = samples_fft_to_spectrum(
            &window,
            self.sample_rate,
            FrequencyLimit::Range(self.min_frequency, self.max_frequency),
            Some(&divide_by_N_sqrt),
        ).ok();

        let mut data_guard = self.spectrum.lock().unwrap();
        *data_guard = spectrum;
    }

    fn smooth_spatial(&self, bins: &mut Vec<f32>) {
        if let Some(filter) = &self.spatial_filter {
            filter.process(bins);
        }
    }

    fn smooth_temporal(&self, bins: &mut Vec<f32>) {
        if let Some(filter) = &self.temporal_filter {
            if let Ok(mut f) = filter.lock() {
                f.process(bins);
            }
        }
    }

    fn binned_spectrum(&self, num_bins: usize) -> Vec<f32> {
        let mut bins = vec![0.0; num_bins];

        let data_guard = self.spectrum.lock().unwrap();
        let spectrum = match data_guard.as_ref() {
            Some(spectrum) => spectrum,
            None => return bins,
        };

        let max_frequency = spectrum.max_fr().val();
        let min_frequency = spectrum.min_fr().val();
        let log_min_frequency = min_frequency.ln();
        let log_max_frequency = max_frequency.ln();

        for &(freq, mag) in spectrum.data() {
            let freq_val = freq.val().ln();
            let bin_index = f32::floor(
                (freq_val - log_min_frequency) / (log_max_frequency - log_min_frequency)
                * (num_bins - 1) as f32
            ) as usize;

            bins[bin_index] += mag.val();
        }

        bins
    }

    pub fn visualization_data(&self, num_bins: usize) -> Vec<f32> {
        let mut bins = self.binned_spectrum(num_bins);
        self.smooth_spatial(&mut bins);
        self.smooth_temporal(&mut bins);
        
        let max_val = bins.iter().cloned().fold(0./0., f32::max);
        for bin in bins.iter_mut() {
            *bin /= max_val;
        }
        
        bins
    }
}