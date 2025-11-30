use egui::Color32;
use egui_plot::{Bar, BarChart, Plot};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::Visualizer;

use crate::frontend::egui_frontend::UiComponent;

#[derive(Clone, Copy, PartialEq)]
pub struct ControlSettings {
    pub num_bins: usize,
    pub window_size: usize,
    pub color: egui::Color32,
}

impl ControlSettings {
    fn default() -> Self {
        Self {
            num_bins: 50,
            window_size: 2048,
            color: Color32::DARK_BLUE,
        }
    }

    fn update_from_visualizer(&mut self, vis: &Visualizer) {
        self.num_bins = vis.config.num_bins;
        self.window_size = vis.config.window_size;
    }
}

/// Egui application struct for visualizing a spectrum.
pub struct EguiFrontend {
    pub visualizer: Arc<Mutex<Visualizer>>,
    last_bins: Vec<f32>,
    control_settings: ControlSettings,
}

impl EguiFrontend {
    pub fn new(visualizer: Arc<Mutex<Visualizer>>) -> Self {
        Self {
            visualizer,
            last_bins: Vec::new(),
            control_settings: ControlSettings::default(),
        }
    }

    fn plot_spectrum(&self, ui: &mut egui::Ui, bins: Vec<f32>) {
        let bars: Vec<Bar> = bins
            .iter()
            .enumerate()
            .filter_map(|(i, &y)| {
                if y >= 0.0 {
                    Some(
                        Bar::new(i as f64, y as f64)
                            .fill(self.control_settings.color)
                            .width(1.0_f64),
                    )
                } else {
                    None
                }
            })
            .collect();

        Plot::new("fft_plot")
            .include_y(0.0)
            .include_y(1.0)
            .show_axes(false)
            .show_grid(false)
            .cursor_color(Color32::TRANSPARENT)
            .show_x(false)
            .show_y(false)
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(BarChart::new("Visualizer".to_string(), bars));
            });
    }
}

impl eframe::App for EguiFrontend {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Ok(vis) = self.visualizer.lock() {
                let bins = vis.visualization_data();
                self.last_bins = bins.clone();
                self.plot_spectrum(ui, bins);
            } else {
                // on lock error, show last known data
                self.plot_spectrum(ui, self.last_bins.clone());
            }
        });

        egui::Area::new("controls".into())
            .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-20.0, 20.0))
            .show(ctx, |ui| {
                egui::CollapsingHeader::new("Visualizer Controls").show(ui, |ui| {
                    ui.set_width(300.0);

                    if let Ok(vis) = self.visualizer.lock() {
                        self.control_settings.update_from_visualizer(&vis);
                    } else {
                        return;
                    };

                    let mut edited_settings = self.control_settings.clone();

                    edited_settings.ui(ui);
                    ui.separator();

                    if let Ok(mut vis) = self.visualizer.lock() {
                        vis.config.filter_manager.ui(ui);
                    } else {
                        return;
                    };

                    ui.separator();

                    let changed = edited_settings != self.control_settings;

                    if changed {
                        self.control_settings = edited_settings;
                        if let Ok(mut vis) = self.visualizer.lock() {
                            if vis.config.num_bins != edited_settings.num_bins {
                                vis.config.set_num_bins(edited_settings.num_bins);
                            }
                            if vis.config.window_size != edited_settings.window_size {
                                vis.config.set_window_size(edited_settings.window_size);
                            }
                            let layout = vis.config.layout.clone();
                            vis.config.filter_manager.update_layout(layout);
                        }
                    }
                });
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
            Box::new(move |_| Ok(Box::new(EguiFrontend::new(visualizer.clone())))),
        )
        .ok();
    }
}
