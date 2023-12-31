use super::super::physics::integrator::Physics;
use super::super::collision::collider::CircleCollider;
use super::super::collision::cache::CollisionCache;
use bevy::ecs::system::{Query, Res};
use bevy::transform::components::Transform;
use bevy::math::Vec3;
use rand::random;

pub(in super::super) fn fluid_resolver(
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

        let resolve_direction = separation_vector.normalize();
        let resolve_vector = overlap * resolve_direction;
        transform_b.translation += resolve_vector;
        transform_a.translation -= resolve_vector;

        if overlap >= 0.2 {
            physics_a.prev_pos = transform_a.translation.truncate();
            physics_b.prev_pos = transform_b.translation.truncate();
        }
    }
}