use super::super::physics::integrator::Physics;
use super::super::collision::collider::CircleCollider;
use super::super::collision::cache::CollisionCache;
use bevy::ecs::system::{Query, Res, Resource};
use bevy::transform::components::Transform;
use bevy::math::{Vec2, Vec3};
use rand::random;

#[derive(Resource)]
pub(in super::super) struct ObjectBoundary {
    lower_bound: Vec3,
    upper_bound: Vec3
}

impl ObjectBoundary {
    pub(in super::super) fn new(center: Vec2, bounds: Vec2) -> Self {
        Self {
            lower_bound: (center - bounds / 2.0).extend(0.0),
            upper_bound: (center + bounds / 2.0).extend(0.0)
        }
    }
}

pub(in super::super) fn basic_transform_resolve_phase(
    mut query: Query<(&mut Transform, &mut Physics, &CircleCollider)>,
    bounds: Res<ObjectBoundary>,
    cache: Res<CollisionCache>
) {
    let CollisionCache {
        contact_pairs_map,
        ..
    } = cache.as_ref();

    for (pair, _) in contact_pairs_map.iter() {
        let [
            (mut transform_a, mut physics_a, collidable_a),
            (mut transform_b, mut physics_b, collidable_b)
        ] = match query.get_many_mut([pair.entity_a, pair.entity_b]) {
            Ok(entities) => entities,
            Err(_) => continue
        };

        let mut separation_vector = transform_b.translation - transform_a.translation;
        let overlap = (collidable_a.radius + collidable_b.radius - separation_vector.length()) / 2.0;

        if overlap <= f32::EPSILON {
            separation_vector = Vec3::new(2.0 * random::<f32>() - 1.0, 2.0 * random::<f32>() - 1.0, 0.0);
        }

        let resolve_direction = separation_vector.normalize();
        let resolve_vector = overlap * resolve_direction;
        transform_b.translation += resolve_vector;
        transform_a.translation -= resolve_vector;

        if overlap >= 2.0 {
            physics_a.prev_pos = transform_a.translation.truncate();
            physics_b.prev_pos = transform_b.translation.truncate();
        }
    }

    for entity in query.iter_mut() {
        let (mut transform, mut physics, _) = entity;
        let mut velocity = transform.translation.truncate() - physics.prev_pos;
        let cache_transform = transform.translation.truncate();
        transform.translation = transform.translation.clamp(bounds.lower_bound, bounds.upper_bound);

        if cache_transform.x < bounds.lower_bound.x || cache_transform.x > bounds.upper_bound.x {
            velocity.x *= -1.0;
        } else if cache_transform.y < bounds.lower_bound.y || cache_transform.y > bounds.upper_bound.y {
            velocity.y *= -1.0;
        }

        physics.prev_pos = transform.translation.truncate() - velocity;
    }
}

/*
fn resolve_phase(
    query: &mut Query<(&mut Transform, &mut Physics, &CircleCollider)>,
    cache: &mut ResMut<CollisionCache>
) {
    let CollisionCache {
        live_pairs,
        bounding_size,
        time_step,
        ..
    } = cache.as_mut();

    for (pair, metadata) in live_pairs.iter_mut() {
        let [
            (mut transform_a, mut physics_a, collidable_a),
            (mut transform_b, mut physics_b, collidable_b)
        ] = match query.get_many_mut([pair.entity_a, pair.entity_b]) {
            Ok(entities) => entities,
            Err(_) => continue
        };

        let mut separation_vector = transform_b.translation - transform_a.translation;
        let are_colliding = transform_a.translation.distance(transform_b.translation) < collidable_a.radius + collidable_b.radius;

        if !are_colliding {
            continue;
        }

        let overlap = (collidable_a.radius + collidable_b.radius - separation_vector.length()) / 2.0;

        if overlap <= f32::EPSILON {
            separation_vector = Vec3::new(2.0 * random::<f32>() - 1.0, 2.0 * random::<f32>() - 1.0, 0.0);
        }

        // TODO: Am I calculating speed wrong? dx / dt
        // Can compare against r*r instead of sqrt
        // No conservation
        // Has the issue of 'popping' when two items overlap too much (due to not setting prev position)
        /*
        let resolve_direction = separation_vector.normalize();
        let resolve_vector = overlap * resolve_direction;
        transform_b.translation += resolve_vector;
        transform_a.translation -= resolve_vector;
        */

        // No conservation
        // Has the issue of 'popping' when two items overlap too much (due to not setting prev position)
        let resolve_direction = separation_vector.normalize();
        let resolve_vector = overlap * resolve_direction;
        transform_b.translation += resolve_vector;
        transform_a.translation -= resolve_vector;

        // Conservation of speed
        // Appears to also suffer from 'stickiness' but less so
        /*
        let speed_a = (transform_a.translation.truncate() - physics_a.prev_pos).length();
        let speed_b = (transform_b.translation.truncate() - physics_b.prev_pos).length();

        let resolve_direction = separation_vector.normalize();
        let resolve_vector = overlap * resolve_direction;
        transform_b.translation += resolve_vector;
        physics_b.prev_pos = (transform_b.translation - speed_b * resolve_direction).truncate();
        transform_a.translation -= resolve_vector;
        physics_a.prev_pos = (transform_a.translation + speed_a * resolve_direction).truncate();
        */

        // Momentum based conservation
        // Suffers from 'stickiness' if we have a decay factor
        /*
        let velocity_a = transform_a.translation.truncate() - physics_a.prev_pos;
        let velocity_b = transform_b.translation.truncate() - physics_b.prev_pos;

        let resolve_direction = separation_vector.normalize();
        let resolve_vector = overlap * resolve_direction;
        transform_b.translation += resolve_vector;
        transform_a.translation -= resolve_vector;

        let diff_a = transform_a.translation.truncate() - transform_b.translation.truncate();
        let deflection_n_a = (2.0 * physics_b.mass) * (velocity_a - velocity_b).dot(diff_a);
        let deflection_d_a = diff_a.length_squared() * (physics_a.mass + physics_b.mass);
        let new_velocity_a = velocity_a - (deflection_n_a / deflection_d_a * diff_a);

        let diff_b = -diff_a;
        let deflection_n_b = (2.0 * physics_a.mass) * (velocity_b - velocity_a).dot(diff_b);
        let deflection_d_b = diff_b.length_squared() * (physics_a.mass + physics_b.mass);
        let new_velocity_b = velocity_b - (deflection_n_b / deflection_d_b * diff_b);

        physics_a.prev_pos = transform_a.translation.truncate() - 1.00 * new_velocity_a;
        physics_b.prev_pos = transform_b.translation.truncate() - 1.00 * new_velocity_b;
        */

        if metadata.is_first_frame {
            metadata.is_first_frame = false;
        }
    }

    let lower = (-1.0 * *bounding_size / 2.0).extend(0.0);
    let upper = (*bounding_size / 2.0).extend(0.0);

    for entity in query.iter_mut() {
        let (mut transform, mut physics, _) = entity;
        let mut velocity = transform.translation.truncate() - physics.prev_pos;
        let cache_transform = transform.translation.truncate();
        transform.translation = transform.translation.clamp(lower, upper);

        if cache_transform.x < lower.x || cache_transform.x > upper.x {
            velocity.x *= -1.0;
        } else if cache_transform.y < lower.y || cache_transform.y > upper.y {
            velocity.y *= -1.0;
        }

        physics.prev_pos = transform.translation.truncate() - velocity;
    }
}
*/