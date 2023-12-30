use super::collider::CircleCollider;
use super::cache::CollisionCache;
use super::contact::ContactMetadata;
use bevy::ecs::system::{Query, ResMut};
use bevy::transform::components::Transform;

pub(in super::super) fn narrow_phase(
    query: Query<(&Transform, &CircleCollider)>,
    mut cache: ResMut<CollisionCache>
) {
    let CollisionCache {
        potential_pairs,
        contact_pairs_map,
        ..
    } = cache.as_mut();
    
    for (_, metadata) in contact_pairs_map.iter_mut() {
        metadata.is_first_frame = false;
        metadata.invalidated = true;
    }

    for pair in potential_pairs.drain(..) {
        let [
            (transform_a, collidable_a),
            (transform_b, collidable_b)
        ] = match query.get_many([pair.entity_a, pair.entity_b]) {
            Ok(entity) => entity,
            Err(_) => continue
        };

        let are_colliding = transform_a.translation.truncate().distance(transform_b.translation.truncate()) < collidable_a.radius + collidable_b.radius;

        if are_colliding {
            match contact_pairs_map.get_mut(&pair) {
                Some(metadata) => {
                    metadata.invalidated = false;
                },
                None => {
                    contact_pairs_map.insert(pair, ContactMetadata {
                        is_first_frame: true,
                        invalidated: false
                    });
                }
            }
        }
    }

    contact_pairs_map.retain(|_, metadata|
        !metadata.invalidated
    );
}