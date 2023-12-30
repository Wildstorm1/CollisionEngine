use super::cache::CollisionCache;
use super::collider::AxisAlignedBoundingBox;
use super::contact::ContactPair;
use bevy::ecs::system::{Query, ResMut};
use bevy::ecs::entity::Entity;
use bevy::log::warn;
use bevy::transform::components::Transform;

pub(in super::super) fn brute_force_broad_phase(
    query: Query<(Entity, &AxisAlignedBoundingBox, &Transform)>,
    mut cache: ResMut<CollisionCache>
) {
    let CollisionCache {
        potential_pairs,
        ..
    } = cache.as_mut();

    for [
        (entity_a, aabb_a, transform_a),
        (entity_b, aabb_b, transform_b)
    ] in query.iter_combinations() {
        if aabb_a.overlaps(transform_a, aabb_b, transform_b) {
            potential_pairs.push(ContactPair::new(entity_a, entity_b));
        }
    }
}

pub(in super::super) fn accelerated_broad_phase(
    query: Query<(Entity, &AxisAlignedBoundingBox, &Transform)>,
    mut cache: ResMut<CollisionCache>
) {
    let CollisionCache {
        accelerator,
        potential_pairs,
        ..
    } = cache.as_mut();

    let accelerator = match accelerator {
        Some(accelerator) => accelerator,
        None => {
            panic!("No spatial accelerator found in collision system")
        }
    };

    for (entity, aabb, transform) in query.iter() {
        match accelerator.update(entity, transform, aabb) {
            Err(e) => {
                warn!("Error updating entity {}", e);
            },
            _ => {}
        }
    }

    for (entity_a, aabb_a, transform_a) in query.iter() {
        for entity_b in accelerator.query(transform_a, aabb_a) {
            if entity_a == entity_b {
                continue;
            }

            let (_, aabb_b, transform_b) = match query.get(entity_b) {
                Ok(entity) => entity,
                Err(_) => continue
            };

            if aabb_a.overlaps(transform_a, aabb_b, transform_b) {
                potential_pairs.push(ContactPair::new(entity_a, entity_b));
            }
        }
    }
}