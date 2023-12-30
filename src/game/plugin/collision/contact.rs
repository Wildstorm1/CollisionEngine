use std::hash::{Hash, Hasher};
use std::ops::BitXor;
use bevy::ecs::entity::Entity;

#[derive(Debug, Copy, Clone)]
pub(in super::super) struct ContactPair {
    pub entity_a: Entity,
    pub entity_b: Entity
}

pub (in super::super) struct ContactMetadata {
    pub is_first_frame: bool,
    pub invalidated: bool
}

impl Hash for ContactPair {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entity_a.to_bits().bitxor(self.entity_b.to_bits()).hash(state);
    }
}

impl PartialEq for ContactPair {
    fn eq(&self, other: &Self) -> bool {
        (self.entity_a == other.entity_a && self.entity_b == other.entity_b) ||
        (self.entity_a == other.entity_b && self.entity_b == other.entity_a)
    }
}

impl Eq for ContactPair {}

impl ContactPair {
    pub(super) fn new(entity_a: Entity, entity_b: Entity) -> Self {
        Self {
            entity_a,
            entity_b
        }
    }
}