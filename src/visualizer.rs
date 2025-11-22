use std::sync::{Arc, Mutex};

use spectrum_analyzer::scaling::divide_by_N_sqrt;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{FrequencyLimit, FrequencySpectrum, samples_fft_to_spectrum};

use crate::filters::{
    AWeightingFilter, AttackReleaseFilter, BinLayout, GaussianFilter, SpatialFilter, TemporalFilter,
};

pub struct Visualizer {
    spectrum: Option<FrequencySpectrum>,
    pub sample_rate: u32,
    pub window_size: usize,
    pub num_bins: usize,
    base_min_freq: f32,
    min_freq: f32,
    max_freq: f32,
    pub spatial_filters: Vec<Arc<Mutex<dyn SpatialFilter>>>,
    pub temporal_filters: Vec<Arc<Mutex<dyn TemporalFilter>>>,
    layout: BinLayout,
    window_rms: f32,
    rms_reference: f32,
    rms_floor: f32,
    rms_gamma: f32,
}

impl Visualizer {
    /// Default constructor. Frequency limits set to 20 Hz - Nyquist.
    pub fn new(sample_rate: u32, window_size: usize, num_bins: usize) -> Self {
        let min_freq = 20.0;
        let max_freq = sample_rate as f32 / 2.0;
        let layout = Self::build_layout(num_bins, min_freq, max_freq, true);
        let visualizer = Self {
            spectrum: None,
            sample_rate,
            window_size,
            num_bins,
            base_min_freq: min_freq,
            min_freq,
            max_freq,
            spatial_filters: vec![
                Arc::new(Mutex::new(AWeightingFilter::new())),
                Arc::new(Mutex::new(GaussianFilter::new(3.0, 2, 3))),
            ],
            temporal_filters: vec![Arc::new(Mutex::new(AttackReleaseFilter::new(0.7, 0.9)))],
            layout: layout,
            window_rms: 0.0,
            rms_reference: 0.6,
            rms_floor: 0.01,
            rms_gamma: 0.7,
        };
        visualizer.refresh_layout_filters();
        visualizer
    }

    fn build_layout(num_bins: usize, min_freq: f32, max_freq: f32, log: bool) -> BinLayout {
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

    fn refresh_layout_filters(&self) {
        for filter in &self.spatial_filters {
            if let Ok(mut f) = filter.lock() {
                f.on_layout_change(&self.layout);
            }
        }
    }

    pub fn update_spectrum(&mut self, samples: &[f32]) {
        let window = hann_window(&samples[0..self.window_size]);
        let mut rms = 0.0_f32;
        for &x in samples.iter().take(self.window_size) {
            rms += x * x;
        }
        rms /= self.window_size as f32;
        rms = rms.sqrt();
        self.window_rms = rms;

        let spectrum = samples_fft_to_spectrum(
            &window,
            self.sample_rate,
            FrequencyLimit::Range(self.min_freq, self.max_freq),
            Some(&divide_by_N_sqrt),
        )
        .ok();

        self.spectrum = spectrum;
    }

    pub fn set_num_bins(&mut self, num_bins: usize) {
        self.num_bins = num_bins.max(1);
        self.layout = Self::build_layout(self.num_bins, self.min_freq, self.max_freq, true);
        self.refresh_layout_filters();
    }

    pub fn set_window_size(&mut self, window_size: usize) {
        if window_size == self.window_size {
            return;
        }
        self.window_size = window_size.max(1);
        // clear current spectrum so stale data not reused
        self.spectrum = None;
        // reset temporal filters (length changes)
        for tf in &self.temporal_filters {
            if let Ok(mut f) = tf.lock() {
                f.reset();
            }
        }

        // Update min_freq adaptively
        let resolution = self.sample_rate as f32 / self.window_size as f32;
        // Factor 2.0 → require ~2 FFT bins before first visual bin.
        let dyn_min = resolution * 2.0;
        let new_min = self.base_min_freq.max(dyn_min);
        if (new_min - self.min_freq).abs() > 0.1 {
            self.min_freq = new_min;
            self.layout = Self::build_layout(self.num_bins, self.min_freq, self.max_freq, true);
            self.refresh_layout_filters();
        }
    }

    pub fn add_spatial_filter<F: SpatialFilter + 'static>(&mut self, f: F) {
        let entry = Arc::new(Mutex::new(f));
        self.spatial_filters.push(entry);
        self.spatial_filters
            .sort_by_key(|sf| sf.lock().ok().map(|r| r.priority()).unwrap_or(100));
        self.refresh_layout_filters();
    }

    fn apply_spatial_filters(&self, bins: &mut Vec<f32>) {
        for filter in &self.spatial_filters {
            if let Ok(f) = filter.lock() {
                f.process(bins);
            }
        }
    }

    fn apply_temporal_filters(&self, bins: &mut Vec<f32>) {
        for filter in &self.temporal_filters {
            if let Ok(mut f) = filter.lock() {
                f.process(bins);
            }
        }
    }

    fn apply_norm(&self, bins: &mut Vec<f32>) {
        for p in bins.iter_mut() {
            *p = p.sqrt();
        }
        let peak = bins.iter().fold(0.0_f32, |m, v| m.max(*v)).max(1e-12);
        let window_rms = self.window_rms;
        let loudness = if window_rms <= self.rms_floor {
            0.0
        } else {
            let norm = (window_rms - self.rms_floor) / (self.rms_reference - self.rms_floor);
            norm.clamp(0.0, 1.0).powf(self.rms_gamma)
        };

        let scale = loudness.max(0.001) / peak;
        for b in bins.iter_mut() {
            *b = (*b * scale).min(1.0);
        }
    }

    fn binned_spectrum(&self, num_bins: usize) -> Vec<f32> {
        let mut bins = vec![0.0; num_bins];

        let spectrum = match self.spectrum.as_ref() {
            Some(spectrum) => spectrum,
            None => return bins,
        };

        let layout = self.layout.clone();

        for &(freq, mag) in spectrum.data() {
            let freq_val = freq.val().ln();
            if freq_val < layout.log_min || freq_val > layout.log_max {
                continue;
            }
            let bin_index = f32::floor(
                (freq_val - layout.log_min) / (layout.log_max - layout.log_min)
                    * (num_bins - 1) as f32,
            ) as usize;

            if bin_index < num_bins {
                bins[bin_index] += mag.val() * mag.val();
            }
        }

        bins
    }

    pub fn visualization_data(&self) -> Vec<f32> {
        let mut bins = self.binned_spectrum(self.num_bins);
        self.apply_spatial_filters(&mut bins);
        self.apply_temporal_filters(&mut bins);
        self.apply_norm(&mut bins);

        bins
    }
}
