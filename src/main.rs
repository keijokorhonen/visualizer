use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use visualizer::{FrontendKind, Visualizer, make_frontend};

mod load_audio;
use load_audio::load_samples_from_file;

fn main() {
    let window_size = 2048;
    let num_bins = 50;

    // Load audio samples from file provided as command line argument
    let (samples, sample_rate) = {
        let args: Vec<String> = std::env::args().collect();
        let path = args.get(1).expect("file path not provided");
        load_samples_from_file(path)
    };

    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let config = device.default_output_config().unwrap();

    // Initialize FFT data
    let visualizer = Arc::new(Mutex::new(Visualizer::new(
        sample_rate,
        window_size,
        num_bins,
    )));
    let visualizer_cb = visualizer.clone();

    let channels = config.channels() as usize;
    let mut sample_pos = 0;

    let stream = device
        .build_output_stream(
            &config.into(),
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Fill output buffer with audio samples
                // Write one frame at a time (respect channel count)
                for frame in output.chunks_mut(channels) {
                    let sample = if sample_pos < samples.len() {
                        samples[sample_pos]
                    } else {
                        0.0
                    };
                    // Duplicate mono sample to all channels
                    for ch in 0..channels {
                        frame[ch] = sample;
                    }
                    sample_pos += 1; // advance per frame, not per channel sample
                }

                // For visualization: process FFT on current window
                if let Ok(mut vis) = visualizer_cb.lock() {
                    let window_size_cur = vis.config.window_size;
                    if sample_pos >= window_size_cur {
                        let start = sample_pos - window_size_cur;
                        let window_samples = &samples[start..sample_pos];
                        vis.update_spectrum(&window_samples);
                    }
                }
            },
            move |err| {
                eprintln!("Stream error: {}", err);
            },
            None,
        )
        .unwrap();

    stream.play().unwrap();

    let frontend = make_frontend(FrontendKind::Egui, visualizer);
    frontend.run();
}
