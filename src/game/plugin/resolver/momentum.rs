use super::super::physics::integrator::Physics;
use super::super::collision::collider::CircleCollider;
use super::super::collision::cache::CollisionCache;
use bevy::ecs::system::{Query, Res};
use bevy::transform::components::Transform;
use bevy::math::Vec3;
use rand::random;

pub(in super::super) fn conserve_momentum_resolver(
    mut query: Query<(&mut Transform, &mut Physics, &CircleCollider)>,
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

        physics_a.prev_pos = transform_a.translation.truncate() - new_velocity_a;
        physics_b.prev_pos = transform_b.translation.truncate() - new_velocity_b;
    }
}