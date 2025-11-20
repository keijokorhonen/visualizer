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
    AttackReleaseFilter,
    BinLayout
};

#[derive(Clone)]
pub struct Visualizer {
    spectrum: Arc<Mutex<Option<FrequencySpectrum>>>,
    pub sample_rate: u32,
    pub window_size: usize,
    pub num_bins: usize,
    min_freq: f32,
    max_freq: f32,
    spatial_filters: Vec<Arc<Mutex<dyn SpatialFilter>>>,
    temporal_filters: Vec<Arc<Mutex<dyn TemporalFilter>>>,
    layout: Arc<Mutex<BinLayout>>,
    window_rms: Arc<Mutex<f32>>,
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
            spectrum: Arc::new(Mutex::new(None)),
            sample_rate,
            window_size,
            num_bins,
            min_freq,
            max_freq,
            spatial_filters: vec![
                // Arc::new(AWeightingFilter::new()),
                Arc::new(Mutex::new(GaussianFilter::new(3.0, 2, 3))),
                ],
            temporal_filters: vec![Arc::new(Mutex::new(AttackReleaseFilter::new(0.7, 0.9)))],
            layout: Arc::new(Mutex::new(layout)),
            window_rms: Arc::new(Mutex::new(0.0)),
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
        BinLayout { centers, min_freq, max_freq, log_min, log_max, spacing_log: log }
    }

    fn refresh_layout_filters(&self) {
        let layout = self.layout.lock().unwrap();
        for filter in &self.spatial_filters {
            if let Ok(mut f) = filter.lock() {
                f.on_layout_change(&layout);
            }
        }
    }

    pub fn update_spectrum(&self, samples: &[f32]) {
        let window = hann_window(&samples[0..self.window_size]);
        let mut rms = 0.0_f32;
        for &x in samples.iter().take(self.window_size) {
            rms += x * x;
        }
        rms /= self.window_size as f32;
        rms = rms.sqrt();
        *self.window_rms.lock().unwrap() = rms;

        let spectrum = samples_fft_to_spectrum(
            &window,
            self.sample_rate,
            FrequencyLimit::Range(self.min_freq, self.max_freq),
            Some(&divide_by_N_sqrt),
        ).ok();

        let mut data_guard = self.spectrum.lock().unwrap();
        *data_guard = spectrum;
    }

    pub fn set_num_bins(&mut self, num_bins: usize) {
        self.num_bins = num_bins.max(1);
        *self.layout.lock().unwrap() = Self::build_layout(
            self.num_bins,
            self.min_freq,
            self.max_freq,
            true
        );
        self.refresh_layout_filters();
    }

    pub fn add_spatial_filter<F: SpatialFilter + 'static>(&mut self, f: F) {
        let entry = Arc::new(Mutex::new(f));
        self.spatial_filters.push(entry);
        self.spatial_filters.sort_by_key(|sf| {
            sf.lock().ok().map(|r| r.priority()).unwrap_or(100)
        });
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
        for p in bins.iter_mut() { *p = p.sqrt(); }
        let peak = bins.iter().fold(0.0_f32, |m, v| m.max(*v)).max(1e-12);
        let window_rms = *self.window_rms.lock().unwrap();
        let loudness = if window_rms <= self.rms_floor {
            0.0
        } else {
            let norm = (window_rms - self.rms_floor) / (self.rms_reference - self.rms_floor);
            norm.clamp(0.0, 1.0).powf(self.rms_gamma)
        };

        let scale = loudness / peak;
        for b in bins.iter_mut() {
            *b = (*b * scale).min(1.0);
        }
    }

    fn binned_spectrum(&self, num_bins: usize) -> Vec<f32> {
        let mut bins = vec![0.0; num_bins];

        let data_guard = self.spectrum.lock().unwrap();
        let spectrum = match data_guard.as_ref() {
            Some(spectrum) => spectrum,
            None => return bins,
        };

        let layout = self.layout.lock().unwrap().clone();

        for &(freq, mag) in spectrum.data() {
            let freq_val = freq.val().ln();
            if freq_val < layout.log_min || freq_val > layout.log_max { continue; }
            let bin_index = f32::floor(
                (freq_val - layout.log_min) / (layout.log_max - layout.log_min)
                * (num_bins - 1) as f32
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