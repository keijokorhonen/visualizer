pub mod egui_vis;
// pub mod bevy_vis;

pub use egui_vis::EguiFrontend;
// pub use bevy_vis::BevyApp;

use crate::Visualizer;

use std::sync::{Arc, Mutex};

pub trait VisualizerFrontend {
    fn run(&self);
}

pub enum FrontendKind {
    Egui,
    // Bevy,
}

pub fn make_frontend(
    kind: FrontendKind,
    visualizer: Arc<Mutex<Visualizer>>,
) -> Box<dyn VisualizerFrontend> {
    match kind {
        FrontendKind::Egui => Box::new(EguiFrontend::new(visualizer)),
        // FrontendKind::Bevy => Box::new(bevy_vis::BevyFrontend::new(visualizer)),
    }
}
