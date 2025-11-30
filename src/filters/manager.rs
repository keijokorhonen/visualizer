use crate::filters::{BinLayout, SpatialFilter, TemporalFilter};
use std::any::TypeId;
use std::sync::{Arc, Mutex};

pub struct FilterEntry<T: ?Sized> {
    id: usize,
    type_id: TypeId,
    filter: Arc<Mutex<T>>,
}

impl<T: ?Sized> FilterEntry<T> {
    pub fn try_lock(&self) -> Option<std::sync::MutexGuard<'_, T>> {
        self.filter.lock().ok()
    }
}

pub struct FilterManager {
    spatial_filters: Vec<FilterEntry<dyn SpatialFilter>>,
    temporal_filters: Vec<FilterEntry<dyn TemporalFilter>>,
    next_id: usize,
}

impl FilterManager {
    pub fn new() -> Self {
        FilterManager {
            spatial_filters: Vec::new(),
            temporal_filters: Vec::new(),
            next_id: 0,
        }
    }

    fn gen_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn spatial_filters(&self) -> &Vec<FilterEntry<dyn SpatialFilter>> {
        &self.spatial_filters
    }

    pub fn temporal_filters(&self) -> &Vec<FilterEntry<dyn TemporalFilter>> {
        &self.temporal_filters
    }

    pub fn active_spatial_filters_types(&self) -> Vec<TypeId> {
        self.spatial_filters
            .iter()
            .filter_map(|entry| Some(entry.type_id))
            .collect()
    }

    pub fn active_temporal_filters_types(&self) -> Vec<TypeId> {
        self.temporal_filters
            .iter()
            .filter_map(|entry| Some(entry.type_id))
            .collect()
    }

    pub fn is_spatial_active_type(&self, tid: TypeId) -> bool {
        self.active_spatial_filters_types().contains(&tid)
    }

    pub fn is_temporal_active_type(&self, tid: TypeId) -> bool {
        self.active_temporal_filters_types().contains(&tid)
    }

    pub fn is_spatial_active<T: SpatialFilter + 'static>(&self) -> bool {
        self.is_spatial_active_type(TypeId::of::<T>())
    }

    pub fn add_spatial_filter<T>(&mut self, filter: T) -> usize
    where
        T: SpatialFilter + 'static,
    {
        let tid = TypeId::of::<T>();
        let id = self.gen_id();
        self.spatial_filters.push(FilterEntry {
            id,
            type_id: tid,
            filter: Arc::new(Mutex::new(filter)),
        });

        id
    }

    pub fn remove_spatial_filter(&mut self, id: usize) {
        if let Some(idx) = self.spatial_filters.iter().position(|e| e.id == id) {
            self.spatial_filters.remove(idx);
        }
    }

    pub fn move_spatial_filter(&mut self, id: usize, index: usize) {
        if let Some(pos) = self.spatial_filters.iter().position(|entry| entry.id == id) {
            let entry = self.spatial_filters.remove(pos);
            let new_index = index.min(self.spatial_filters.len());
            self.spatial_filters.insert(new_index, entry);
        }
    }

    pub fn add_temporal_filter<T>(&mut self, filter: T) -> usize
    where
        T: TemporalFilter + 'static,
    {
        let tid = TypeId::of::<T>();
        let id = self.gen_id();
        self.temporal_filters.push(FilterEntry {
            id,
            type_id: tid,
            filter: Arc::new(Mutex::new(filter)),
        });

        id
    }

    pub fn remove_temporal_filter(&mut self, id: usize) {
        if let Some(idx) = self
            .temporal_filters
            .iter()
            .position(|entry| entry.id == id)
        {
            self.temporal_filters.remove(idx);
        }
    }

    pub fn move_temporal_filter(&mut self, id: usize, index: usize) {
        if let Some(pos) = self
            .temporal_filters
            .iter()
            .position(|entry| entry.id == id)
        {
            let entry = self.temporal_filters.remove(pos);
            let new_index = index.min(self.temporal_filters.len());
            self.temporal_filters.insert(new_index, entry);
        }
    }

    pub fn apply_spatial_filters(&self, samples: &mut [f32]) {
        for entry in &self.spatial_filters {
            if let Ok(filter) = entry.filter.lock() {
                filter.process(samples);
            }
        }
    }

    pub fn apply_temporal_filters(&self, samples: &mut [f32]) {
        for entry in &self.temporal_filters {
            if let Ok(mut filter) = entry.filter.lock() {
                filter.process(samples);
            }
        }
    }

    pub fn refresh_layout(&self, layout: &BinLayout) {
        for entry in &self.spatial_filters {
            if let Ok(mut f) = entry.filter.lock() {
                f.on_layout_change(layout);
            }
        }
    }

    pub fn reset_temporal_filters(&self) {
        for entry in &self.temporal_filters {
            if let Ok(mut filter) = entry.filter.lock() {
                filter.reset();
            }
        }
    }
}
