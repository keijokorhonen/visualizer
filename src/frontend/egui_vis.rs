use egui_plot::{Plot, BarChart, Bar};
use std::time::Duration;

use crate::fft_data::FFTData;

/// Egui application struct for visualizing FFT data.
/// Holds a reference to FFTData.
#[derive(Clone)]
pub struct EguiFrontend {
    pub fft: FFTData,
}

impl EguiFrontend {
    pub fn new(fft: FFTData) -> Self { Self { fft } }

    fn plot_spectrum(&self, ui: &mut egui::Ui) {
        let bins= self.fft.visualization_data();
        let bars: Vec<Bar> = bins.iter().enumerate()
            .filter_map(|(i, &y)| {
                if y >= 0.0 {
                    Some(
                        Bar::new(i as f64, y as f64)
                        .fill(egui::Color32::DARK_BLUE)
                        .width((1.0) as f64)
                    )
                } else {
                    println!("Skipping zero magnitude {:.2} at bin {}", y, i);
                    None
                }
            }
        ).collect();
        
        Plot::new("fft_plot")
        .include_y(0.0)
        .include_y(1.0)
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(
                BarChart::new(String::from("FFT"),
                bars)
            );
        });
    }
}

impl eframe::App for EguiFrontend {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.plot_spectrum(ui);
        });
        // Request repaint at approx 60 FPS
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

impl crate::frontend::VisualizerFrontend for EguiFrontend {
    fn run(&self) {
        eframe::run_native(
            "Visualizer (egui)",
            eframe::NativeOptions::default(),
            Box::new(move |_| {
                Ok(Box::new(self.clone()))
            })
        ).ok();
    }
}