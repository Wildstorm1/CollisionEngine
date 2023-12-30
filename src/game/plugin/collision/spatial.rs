use std::fmt::{Debug, Display, Formatter, Error};
use bevy::ecs::entity::Entity;
use bevy::transform::components::Transform;
use super::collider::AxisAlignedBoundingBox;

#[derive(Debug, Clone)]
pub struct AcceleratorError {
    pub(super) entity: Entity,
    pub(super) aabb: Option<AxisAlignedBoundingBox>,
    pub(super) pos: Option<Transform>
}

impl Display for AcceleratorError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if self.aabb.is_none() && self.pos.is_none() {
            write!(f, "Invalid entity {:?}", self.entity)
        } else if self.aabb.is_some() {
            write!(f, "Invalid entity {:?} or AABB {:?}", self.entity, self.aabb)
        } else if self.pos.is_some() {
            write!(f, "Invalid entity {:?} or position {:?}", self.entity, self.pos)
        } else {
            write!(
                f,
                "Invalid entity {:?}, AABB {:?}, or position {:?}",
                self.entity,
                self.aabb,
                self.pos
            )
        }
    }
}

pub trait SpatialAccelerator {
    fn insert(&mut self, id: Entity, pos: &Transform, aabb: &AxisAlignedBoundingBox) -> Result<(), AcceleratorError>;
    fn update(&mut self, id: Entity, pos: &Transform, aabb: &AxisAlignedBoundingBox) -> Result<(), AcceleratorError>;
    fn remove(&mut self, id: Entity) -> Result<(), AcceleratorError>;
    fn query<'a>(&'a self, pos: &Transform, aabb: &AxisAlignedBoundingBox) -> Box<dyn Iterator<Item = Entity> + 'a>;
}