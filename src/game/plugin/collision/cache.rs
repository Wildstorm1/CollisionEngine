use super::spatial::SpatialAccelerator;
use super::contact::{ContactPair, ContactMetadata};
use std::collections::HashMap;
use bevy::ecs::system::Resource;

#[derive(Resource)]
pub(in super::super) struct CollisionCache {
    pub accelerator: Option<Box<dyn SpatialAccelerator + Send + Sync>>,
    pub potential_pairs: Vec<ContactPair>,
    pub contact_pairs_map: HashMap<ContactPair, ContactMetadata>
}

impl CollisionCache {
    pub(in super::super) fn new(accelerator: Option<Box<dyn SpatialAccelerator + Send + Sync>>) -> Self {
        Self {
            accelerator: accelerator,
            potential_pairs: Vec::new(),
            contact_pairs_map: HashMap::new()
        }
    }

    pub(in super::super) fn with_capacity(capacity: usize, accelerator: Option<Box<dyn SpatialAccelerator + Send + Sync>>) -> Self {
        Self {
            accelerator: accelerator,
            potential_pairs: Vec::with_capacity(capacity),
            contact_pairs_map: HashMap::with_capacity(capacity)
        }
    }
}