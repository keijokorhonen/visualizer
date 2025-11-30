use spectrum_analyzer::scaling::divide_by_N_sqrt;
use spectrum_analyzer::windows::hann_window;
use spectrum_analyzer::{FrequencyLimit, FrequencySpectrum, samples_fft_to_spectrum};

use crate::filters::{
    AttackReleaseFilter, BinLayout, FilterManager, GaussianFilter, SpatialFilter, TemporalFilter,
};

pub struct VisualizerConfig {
    pub sample_rate: u32,
    pub window_size: usize,
    pub num_bins: usize,
    base_min_freq: f32,
    min_freq: f32,
    max_freq: f32,
    pub filter_manager: FilterManager,
    layout: BinLayout,
    window_rms: f32,
    rms_reference: f32,
    rms_floor: f32,
    rms_gamma: f32,
}

impl VisualizerConfig {
    pub fn set_num_bins(&mut self, num_bins: usize) {
        self.num_bins = num_bins.max(1);
        self.layout = BinLayout::build_layout(self.num_bins, self.min_freq, self.max_freq, true);
        self.filter_manager.refresh_layout(&self.layout);
    }

    pub fn set_window_size(&mut self, window_size: usize) {
        if window_size == self.window_size {
            return;
        }
        self.window_size = window_size.max(1);

        // reset temporal filters (length changes)
        self.filter_manager.reset_temporal_filters();

        // Update min_freq adaptively
        let resolution = self.sample_rate as f32 / self.window_size as f32;
        // Factor 2.0 → require ~2 FFT bins before first visual bin.
        let dyn_min = resolution * 2.0;
        let new_min = self.base_min_freq.max(dyn_min);
        if (new_min - self.min_freq).abs() > 0.1 {
            self.min_freq = new_min;
            self.layout =
                BinLayout::build_layout(self.num_bins, self.min_freq, self.max_freq, true);
            self.filter_manager.refresh_layout(&self.layout);
        }
    }

    pub fn add_spatial_filter<T>(&mut self, filter: T) -> usize
    where
        T: SpatialFilter + 'static,
    {
        let id = self.filter_manager.add_spatial_filter(filter);
        // initialize the new filter using current layout
        if let Some(entry) = self.filter_manager.spatial_filters().last() {
            if let Some(mut guard) = entry.try_lock() {
                guard.on_layout_change(&self.layout);
            }
        }
        id
    }

    pub fn add_temporal_filter<T>(&mut self, filter: T) -> usize
    where
        T: TemporalFilter + 'static,
    {
        self.filter_manager.add_temporal_filter(filter)
    }
}

pub struct Visualizer {
    spectrum: Option<FrequencySpectrum>,
    pub config: VisualizerConfig,
}

impl Visualizer {
    /// Default constructor. Frequency limits set to 20 Hz - Nyquist.
    pub fn new(sample_rate: u32, window_size: usize, num_bins: usize) -> Self {
        let min_freq = 20.0;
        let max_freq = sample_rate as f32 / 2.0;
        let layout = BinLayout::build_layout(num_bins, min_freq, max_freq, true);

        let mut filter_manager = FilterManager::new();
        filter_manager.add_spatial_filter(GaussianFilter::new(3.0, 2, 3));
        filter_manager.add_temporal_filter(AttackReleaseFilter::new(0.7, 0.9));

        let config = VisualizerConfig {
            sample_rate,
            window_size,
            num_bins,
            base_min_freq: min_freq,
            min_freq,
            max_freq,
            filter_manager,
            layout,
            window_rms: 0.0,
            rms_reference: 0.6,
            rms_floor: 0.01,
            rms_gamma: 0.7,
        };
        let visualizer = Self {
            spectrum: None,
            config,
        };
        visualizer
            .config
            .filter_manager
            .refresh_layout(&visualizer.config.layout);
        visualizer
    }

    pub fn update_spectrum(&mut self, samples: &[f32]) {
        let window = hann_window(&samples[0..self.config.window_size]);
        let mut rms = 0.0_f32;
        for &x in samples.iter().take(self.config.window_size) {
            rms += x * x;
        }
        rms /= self.config.window_size as f32;
        rms = rms.sqrt();
        self.config.window_rms = rms;

        let spectrum = samples_fft_to_spectrum(
            &window,
            self.config.sample_rate,
            FrequencyLimit::Range(self.config.min_freq, self.config.max_freq),
            Some(&divide_by_N_sqrt),
        )
        .ok();

        self.spectrum = spectrum;
    }

    fn apply_norm(&self, bins: &mut Vec<f32>) {
        for p in bins.iter_mut() {
            *p = p.sqrt();
        }
        let peak = bins.iter().fold(0.0_f32, |m, v| m.max(*v)).max(1e-12);
        let window_rms = self.config.window_rms;
        let loudness = if window_rms <= self.config.rms_floor {
            0.0
        } else {
            let norm = (window_rms - self.config.rms_floor)
                / (self.config.rms_reference - self.config.rms_floor);
            norm.clamp(0.0, 1.0).powf(self.config.rms_gamma)
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

        let layout = self.config.layout.clone();

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
        let mut bins = self.binned_spectrum(self.config.num_bins);
        let filter_manager = &self.config.filter_manager;
        filter_manager.apply_spatial_filters(&mut bins);
        filter_manager.apply_temporal_filters(&mut bins);
        self.apply_norm(&mut bins);

        bins
    }
}
