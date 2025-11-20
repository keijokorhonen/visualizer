pub mod egui_vis;
// pub mod bevy_vis;

pub use egui_vis::EguiFrontend;
// pub use bevy_vis::BevyApp;

use crate::Visualizer;

pub trait VisualizerFrontend {
    fn run(&self);
}

pub enum FrontendKind {
    Egui,
    // Bevy,
}

pub fn make_frontend(kind: FrontendKind, visualizer: Visualizer) -> Box<dyn VisualizerFrontend> {
    match kind {
        FrontendKind::Egui => Box::new(EguiFrontend::new(visualizer)),
        // FrontendKind::Bevy => Box::new(bevy_vis::BevyFrontend::new(visualizer)),
    }
}