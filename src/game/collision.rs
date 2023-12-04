use super::bundles::CollisionEvent;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ops::BitXor;
use std::vec::Vec;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::EventWriter;
use bevy::math::Vec2;
use bevy::ecs::system::Query;
use bevy::prelude::Transform;

// Can we intern these collider structs??

pub struct CircleCollider {
    pub radius: f32
}

#[derive(Component)]
pub struct Collidable {
    pub bounds: Vec2,
    pub collider: CircleCollider
}

struct CollisionPair {
    entity_a: Entity,
    entity_b: Entity
}

impl Hash for CollisionPair {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entity_a.to_bits().bitxor(self.entity_b.to_bits()).hash(state);
    }
}

impl PartialEq for CollisionPair {
    fn eq(&self, other: &Self) -> bool {
        (self.entity_a == other.entity_a && self.entity_b == other.entity_b) ||
        (self.entity_a == other.entity_b && self.entity_b == other.entity_a)
    }
}

impl Eq for CollisionPair {}

pub fn collision_update_step(
    query: Query<(Entity, &Collidable, &Transform)>,
    mut event_writer: EventWriter<CollisionEvent>
) {
    let possible_pairs = broad_phase(&query);
    narrow_phase(&query, event_writer, possible_pairs);
}

fn broad_phase(
    query: &Query<(Entity, &Collidable, &Transform)>
) -> Vec<CollisionPair> {
    let mut seen_pairs: HashSet<CollisionPair> = HashSet::new();

    for (entity_a, collidable_a, transform_a) in query.iter() {
        for (entity_b, collidable_b, transform_b) in query.iter() {
            let pair = CollisionPair {
                entity_a: entity_a,
                entity_b: entity_b
            };

            if entity_a == entity_b || seen_pairs.contains(&pair) {
                continue;
            }

            let half_bound_a = collidable_a.bounds.extend(0.0) / 2.0;
            let half_bound_b = collidable_b.bounds.extend(0.0) / 2.0;
            let a_min = transform_a.translation - half_bound_a;
            let a_max = transform_a.translation + half_bound_a;
            let b_min = transform_b.translation - half_bound_b;
            let b_max = transform_b.translation + half_bound_b;

            if a_min.x > b_max.x || a_max.x < b_min.x || a_min.y > b_max.y || a_max.y < b_min.y {
                continue;
            }

            seen_pairs.insert(pair);
        }
    }

    seen_pairs.into_iter().collect()
}

fn narrow_phase(
    query: &Query<(Entity, &Collidable, &Transform)>,
    mut event_writer: EventWriter<CollisionEvent>,
    pairs: Vec<CollisionPair>
) {
    for pair in pairs {
        let (entity_a, collidable_a, transform_a) = match query.get(pair.entity_a) {
            Ok(entity) => entity,
            Err(_) => continue
        };

        let (entity_b, collidable_b, transform_b) = match query.get(pair.entity_b) {
            Ok(entity) => entity,
            Err(_) => continue
        };

        if transform_a.translation.distance(transform_b.translation) < collidable_a.collider.radius + collidable_b.collider.radius {
            event_writer.send(CollisionEvent {
                entity: entity_a,
                other: entity_b
            });

            event_writer.send(CollisionEvent {
                entity: entity_b,
                other: entity_a
            });
        }
    }
}