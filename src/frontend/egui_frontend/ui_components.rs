use egui::{self, Ui};

use crate::filters::*;
use crate::frontend::egui_frontend::ControlSettings;

pub trait UiComponent {
    fn ui(&mut self, _ui: &mut egui::Ui) {}
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

impl UiComponent for AttackReleaseFilter {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.enabled, "");
            if !self.enabled {
                ui.add_enabled_ui(false, |ui| {
                    ui.label("Attack-Release (disabled)");
                });
                return;
            }
            ui.label("Attack-Release:");
            ui.add(
                egui::DragValue::new(&mut self.attack_alpha)
                    .speed(0.01)
                    .range(0.0..=1.0)
                    .prefix("attack="),
            );
            ui.add(
                egui::DragValue::new(&mut self.release_alpha)
                    .speed(0.01)
                    .range(0.0..=1.0)
                    .prefix("release="),
            );
        });
    }
}

impl UiComponent for PeakHoldDecayFilter {}
impl UiComponent for ExponentialFilter {}

impl UiComponent for ControlSettings {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("General:");
        ui.horizontal(|ui| {
            ui.label("Bins:");
            ui.style_mut().spacing.slider_width = 200.0;
            ui.add(egui::Slider::new(&mut self.num_bins, 8..=256));
        });
        ui.horizontal(|ui| {
            ui.label("Window:");
            for size in [512, 1024, 2048, 4096, 8192] {
                ui.selectable_value(&mut self.window_size, size, size.to_string());
            }
        });
    }
}
