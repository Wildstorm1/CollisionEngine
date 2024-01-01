use super::super::physics::integrator::Physics;
use super::super::collision::collider::CircleCollider;
use bevy::ecs::system::{Resource, Query, Res};
use bevy::math::{Vec2, Vec3};
use bevy::transform::components::Transform;

#[derive(Resource)]
pub(in super::super) struct ObjectBoundary {
    lower_bound: Vec3,
    upper_bound: Vec3,
    decay: f32
}

impl ObjectBoundary {
    pub(in super::super) fn new(center: Vec2, bounds: Vec2, decay: Option<f32>) -> Self {
        Self {
            lower_bound: (center - bounds / 2.0).extend(0.0),
            upper_bound: (center + bounds / 2.0).extend(0.0),
            decay: decay.unwrap_or(1.0)
        }
    }
}

pub(in super::super) fn bound_in_box(
    mut query: Query<(&mut Transform, &mut Physics, &CircleCollider)>,
    bounds: Res<ObjectBoundary>
) {
    for (mut transform, mut physics, collider) in query.iter_mut() {
        let vec_pos = transform.translation.truncate();
        let mut velocity = vec_pos - physics.prev_pos;
        let lower = bounds.lower_bound + collider.radius * Vec3::new(1.0, 1.0, 0.0);
        let upper = bounds.upper_bound - collider.radius * Vec3::new(1.0, 1.0, 0.0);

        if vec_pos.x < lower.x || vec_pos.x > upper.x {
            velocity.x *= -bounds.decay;
        } else if vec_pos.y < lower.y || vec_pos.y > upper.y {
            velocity.y *= -bounds.decay;
        }

        transform.translation = transform.translation.clamp(lower, upper);
        physics.prev_pos = transform.translation.truncate() - velocity;
    }
}