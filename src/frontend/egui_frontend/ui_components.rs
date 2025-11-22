use egui;
use std::sync::{Arc, Mutex};

use crate::Visualizer;
use crate::filters::*;

pub trait UiComponent {
    fn ui(&mut self, ui: &mut egui::Ui) {}
    fn group_name(&self) -> &'static str {
        ""
    }
}

impl UiComponent for GaussianFilter {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let old_sigma = self.sigma;
        let old_radius = self.radius;

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.enabled, "");
            if !self.enabled {
                ui.add_enabled_ui(false, |ui| {
                    ui.label("Gaussian (disabled)");
                });
                return;
            }
            ui.label("Gaussian:");
            ui.add(
                egui::DragValue::new(&mut self.sigma)
                    .speed(0.1)
                    .range(0.1..=20.0)
                    .prefix("σ="),
            );
            ui.add(
                egui::DragValue::new(&mut self.radius)
                    .speed(1)
                    .range(0..=128)
                    .prefix("r="),
            );
            ui.add(
                egui::DragValue::new(&mut self.num_passes)
                    .speed(1)
                    .range(1..=20)
                    .prefix("passes="),
            );
        });

        self.recompute_if_needed(old_sigma, old_radius);
    }

    fn group_name(&self) -> &'static str {
        "Gaussian Filter"
    }
}

impl UiComponent for EqCurveFilter {}

impl UiComponent for AWeightingFilter {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.enabled, "");
            if !self.enabled {
                ui.add_enabled_ui(false, |ui| {
                    ui.label("A-Weighting (disabled)");
                });
                return;
            }
            ui.label("A-Weighting Filter");
        });
    }
}

// Control settings wrapper
pub struct ControlSettingsUi {
    pub num_bins: usize,
    pub window_size: usize,
    vis: Arc<Mutex<Visualizer>>,
}

impl ControlSettingsUi {
    pub fn new(vis: Arc<Mutex<Visualizer>>) -> Self {
        let (num_bins, window_size) = {
            let v = vis.lock().unwrap();
            (v.num_bins, v.window_size)
        };
        Self {
            num_bins,
            window_size,
            vis,
        }
    }
}

impl UiComponent for ControlSettingsUi {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("General:");
        ui.horizontal(|ui| {
            ui.label("Bins:");
            ui.add(egui::Slider::new(&mut self.num_bins, 8..=256));
        });
        ui.horizontal(|ui| {
            ui.label("Window:");
            for size in [256, 512, 1024, 2048, 4096] {
                ui.selectable_value(&mut self.window_size, size, size.to_string());
            }
        });

        if let Ok(mut v) = self.vis.lock() {
            if v.num_bins != self.num_bins {
                v.set_num_bins(self.num_bins);
            }
            if v.window_size != self.window_size {
                v.set_window_size(self.window_size);
            }
        }
    }

    fn group_name(&self) -> &'static str {
        "General Settings"
    }
}
