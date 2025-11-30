use crate::filters::{AWeightingFilter, GaussianFilter, SpatialFilter};
use std::any::TypeId;
use std::sync::{Arc, Mutex};

pub struct SpatialFactory {
    pub type_id: TypeId,
    pub name: &'static str,
    pub make: fn() -> Arc<Mutex<dyn SpatialFilter>>,
}

pub fn spatial_factories() -> Vec<SpatialFactory> {
    vec![
        SpatialFactory {
            type_id: TypeId::of::<GaussianFilter>(),
            name: "Gaussian",
            make: || Arc::new(Mutex::new(GaussianFilter::default())),
        },
        SpatialFactory {
            type_id: TypeId::of::<AWeightingFilter>(),
            name: "A-Weighting",
            make: || Arc::new(Mutex::new(AWeightingFilter::default())),
        },
    ]
}
