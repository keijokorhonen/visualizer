use egui::Color32;
use egui_plot::{Plot, BarChart, Bar};
use std::time::Duration;
use std::sync::{
    Arc, Mutex,
};

use crate::Visualizer;

/// Egui application struct for visualizing a spectrum.
pub struct EguiFrontend {
    pub visualizer: Arc<Mutex<Visualizer>>,
}

impl EguiFrontend {
    pub fn new(visualizer: Arc<Mutex<Visualizer>>) -> Self { Self { visualizer } }
    fn plot_spectrum(&self, ui: &mut egui::Ui, bins: Vec<f32>) {
        let bars: Vec<Bar> = bins.iter().enumerate()
            .filter_map(|(i, &y)| {
                if y >= 0.0 {
                    Some(
                        Bar::new(i as f64, y as f64)
                        .fill(egui::Color32::DARK_BLUE)
                        .width(1.0_f64)
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
        .show_axes(false)
        .show_grid(false)
        .cursor_color(Color32::TRANSPARENT)
        .show_x(false)
        .show_y(false)
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(
                BarChart::new("Visualizer".to_string(),
                bars)
            );
        });
    }
}

impl eframe::App for EguiFrontend {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut num_bins;
            let mut window_size;
            let data = {
                if let Ok(vis) = self.visualizer.lock() {
                    num_bins = vis.num_bins;
                    window_size = vis.window_size;
                    vis.visualization_data()
                } else { return; }
            };

            ui.horizontal(|ui| {
                ui.label("Bins:");
                ui.style_mut().spacing.slider_width = 120.0;
                if ui.add(egui::Slider::new(&mut num_bins, 4..=256)).changed() {
                    if let Ok(mut vis) = self.visualizer.lock() {
                        vis.set_num_bins(num_bins);
                    }
                }
                ui.label("Window size:");
                for size in [256, 512, 1024, 2048, 4096].iter() {
                    ui.selectable_value(&mut window_size, *size, size.to_string());
                }
                if let Ok(mut vis) = self.visualizer.lock() {
                    if window_size != vis.window_size {
                        vis.set_window_size(window_size);
                    }
                }
            });
            
            ui.separator();

            
            self.plot_spectrum(ui, data);
        });
        // Request repaint at approx 60 FPS
        ctx.request_repaint_after(Duration::from_millis(16));
    }
}

impl crate::frontend::VisualizerFrontend for EguiFrontend {
    fn run(&self) {
        let visualizer = Arc::clone(&self.visualizer);
        eframe::run_native(
            "Visualizer (egui)",
            eframe::NativeOptions::default(),
            Box::new(move |_| {
                Ok(Box::new(EguiFrontend::new(visualizer.clone())))
            })
        ).ok();
    }
}