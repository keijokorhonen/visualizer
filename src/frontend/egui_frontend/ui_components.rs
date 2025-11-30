use crate::filters::registry::spatial_factories;
use crate::filters::*;
use crate::frontend::egui_frontend::ControlSettings;
use egui;

pub trait UiComponent {
    fn ui(&mut self, _ui: &mut egui::Ui) {}
    fn group_name(&self) -> &'static str {
        ""
    }
}

fn add_checkbox(ui: &mut egui::Ui, enabled: &mut bool, label: &str) {
    ui.horizontal(|ui| {
        ui.add(egui::Checkbox::without_text(enabled));
        if !*enabled {
            ui.add_enabled_ui(false, |ui| {
                ui.label(format!("{} (disabled)", label));
            });
            return;
        }
    });
}

impl UiComponent for FilterManager {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Spatial Filters:");

        let ids: Vec<usize> = self.spatial_filters().iter().map(|e| e.id).collect();

        for (idx, id) in ids.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    let at_top = idx == 0;
                    let at_bottom = idx + 1 == ids.len();

                    if ui
                        .add_enabled(!at_top, egui::Button::new("⏶").small())
                        .clicked()
                    {
                        self.move_spatial_filter(*id, idx - 1);
                    }
                    if ui
                        .add_enabled(!at_bottom, egui::Button::new("⏷").small())
                        .clicked()
                    {
                        self.move_spatial_filter(*id, idx + 1);
                    }
                });

                if let Some(entry) = self.spatial_filters().iter().find(|e| e.id == *id) {
                    if let Some(mut f) = entry.try_lock() {
                        f.ui(ui);
                    }
                }
            });
        }

        ui.menu_button("Add Filter", |ui| {
            for f in spatial_factories() {
                if self.is_spatial_active_type(f.type_id) {
                    continue;
                }
                if ui.button(f.name).clicked() {
                    if f.name == "Gaussian" {
                        self.add_spatial_filter(GaussianFilter::default());
                        ui.close();
                    }
                    if f.name == "A-Weighting" {
                        self.add_spatial_filter(AWeightingFilter::default());
                        ui.close();
                    }
                }
            }
            if spatial_factories()
                .iter()
                .all(|f| self.is_spatial_active_type(f.type_id))
            {
                ui.label("No more filters");
            }
        });

        ui.separator();

        ui.label("Temporal Filters:");
        for f in self.temporal_filters() {
            if let Some(mut filter) = f.try_lock() {
                filter.ui(ui);
            }
        }
    }

    fn group_name(&self) -> &'static str {
        "Filter Manager"
    }
}

impl UiComponent for GaussianFilter {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let old_sigma = self.sigma;
        let old_radius = self.radius;

        ui.horizontal(|ui| {
            let label = self.group_name();
            add_checkbox(ui, &mut self.enabled, label);
            if !self.enabled {
                return;
            }
            ui.label(format!("{}:", label));
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
        "Gaussian"
    }
}

impl UiComponent for AWeightingFilter {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let label = self.group_name();
            add_checkbox(ui, &mut self.enabled, label);
            if !self.enabled {
                return;
            }
            ui.label(format!("{}", label));
        });
    }

    fn group_name(&self) -> &'static str {
        "A-Weighting"
    }
}

impl UiComponent for AttackReleaseFilter {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let label = self.group_name();
            add_checkbox(ui, &mut self.enabled, label);
            if !self.enabled {
                return;
            }
            ui.label(format!("{}:", label));
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

    fn group_name(&self) -> &'static str {
        "Attack/Release"
    }
}

impl UiComponent for PeakHoldDecayFilter {
    fn group_name(&self) -> &'static str {
        "Peak-Hold & Decay"
    }
}
impl UiComponent for ExponentialFilter {
    fn group_name(&self) -> &'static str {
        "Exponential"
    }
}

impl UiComponent for ControlSettings {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label("General:");
        ui.horizontal(|ui| {
            ui.label("Bins:");
            ui.style_mut().spacing.slider_width = 200.0;
            ui.add(egui::Slider::new(&mut self.num_bins, 8..=512));
        });
        ui.horizontal(|ui| {
            ui.label("Window:");
            for size in [512, 1024, 2048, 4096, 8192] {
                ui.selectable_value(&mut self.window_size, size, size.to_string());
            }
        });
        ui.horizontal(|ui| {
            ui.label("Color:");
            ui.color_edit_button_srgba(&mut self.color);
        });
    }

    fn group_name(&self) -> &'static str {
        "Control Settings"
    }
}
