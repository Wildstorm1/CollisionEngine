use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::math::Vec2;
use bevy::transform::components::Transform;

#[derive(Copy, Clone, Debug, Component)]
pub struct AxisAlignedBoundingBox {
    bounds: Vec2
}

impl AxisAlignedBoundingBox {
    pub fn new(bounds: Vec2) -> Self {
        Self {
            bounds: bounds
        }
    }

    pub fn overlaps(&self, pos: &Transform, other_box: &AxisAlignedBoundingBox, other_pos: &Transform) -> bool {
        let half_bound_a = self.bounds / 2.0;
        let half_bound_b = other_box.bounds / 2.0;
        let translation_a = pos.translation.truncate();
        let translation_b = other_pos.translation.truncate();
        let a_min = translation_a - half_bound_a;
        let a_max = translation_a + half_bound_a;
        let b_min = translation_b - half_bound_b;
        let b_max = translation_b + half_bound_b;

        !(a_min.x > b_max.x || a_max.x < b_min.x || a_min.y > b_max.y || a_max.y < b_min.y)
    }
}

#[derive(Copy, Clone, Debug, Component)]
pub struct CircleCollider {
    pub radius: f32
}

#[derive(Debug, Bundle)]
pub struct CircleColliderBundle {
    pub aabb: AxisAlignedBoundingBox,
    pub collider: CircleCollider
}