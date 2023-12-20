use super::physics::Physics;
use bevy::app::{App, Plugin, Update, PostUpdate};
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::{Event, EventWriter, EventReader};
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::math::Vec2;
use bevy::ecs::system::{Query, ResMut, Resource};
use bevy::prelude::Transform;
use bevy::render::color::Color;
use bevy_prototype_debug_lines::DebugShapes;
use std::hash::{Hash, Hasher};
use std::ops::BitXor;
use std::collections::{HashSet, HashMap};

#[derive(Component)]
pub struct Collider {
    pub bounds: Vec2,
    pub radius: f32
}

#[derive(Event)]
pub struct CollisionBeganEvent {
    pub entity: Entity,
    pub other_position: Vec2,
    pub other_velocity: Vec2,
    pub other_mass: f32,
    pub other_radius: f32
}

#[derive(Event)]
pub struct CollisionUpdateEvent {
    pub entity: Entity,
    pub other_position: Vec2,
    pub other_velocity: Vec2,
    pub other_mass: f32,
    pub other_radius: f32
}

#[derive(Event)]
pub struct CollisionEndedEvent {
    pub entity: Entity,
    pub other_position: Vec2,
    pub other_velocity: Vec2,
    pub other_mass: f32,
    pub other_radius: f32
}

#[derive(Resource)]
struct CollisionCache {
    potential_pairs: HashSet<CollisionPair>,
    live_pairs: HashSet<CollisionPair>
}

pub struct CollisionPlugin {
    pub enable_debug: bool
}

// TODO: How do we handle the fact that this depends on another plugin?
// There are other 'life cycle' methods
impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        let debug_flag = self.enable_debug;

        app.add_event::<CollisionBeganEvent>()
            .add_event::<CollisionUpdateEvent>()
            .add_event::<CollisionEndedEvent>()
            .insert_resource(CollisionCache {
                potential_pairs: HashSet::with_capacity(200),
                live_pairs: HashSet::with_capacity(100)
            })       
            .add_systems(Update, collision_update_step)
            .add_systems(PostUpdate, debug_collision_update_step.run_if(move || debug_flag));
    }
}

impl Default for CollisionPlugin {
    fn default() -> Self {
        CollisionPlugin {
            enable_debug: false
        }
    }
}

trait EventConstructor {
    fn new(
        entity: Entity,
        other_transform: &Transform,
        other_physics: &Physics,
        other_collider: &Collider
    ) -> Self;
}

impl EventConstructor for CollisionBeganEvent {
    fn new(
        entity: Entity,
        other_transform: &Transform,
        other_physics: &Physics,
        other_collider: &Collider
    ) -> Self {
        CollisionBeganEvent {
            entity: entity,
            other_position: other_transform.translation.truncate(),
            other_velocity: other_physics.velocity,
            other_mass: other_physics.mass,
            other_radius: other_collider.radius
        }
    }
}

impl EventConstructor for CollisionUpdateEvent {
    fn new(
        entity: Entity,
        other_transform: &Transform,
        other_physics: &Physics,
        other_collider: &Collider
    ) -> Self {
        CollisionUpdateEvent {
            entity: entity,
            other_position: other_transform.translation.truncate(),
            other_velocity: other_physics.velocity,
            other_mass: other_physics.mass,
            other_radius: other_collider.radius
        }
    }
}

impl EventConstructor for CollisionEndedEvent {
    fn new(
        entity: Entity,
        other_transform: &Transform,
        other_physics: &Physics,
        other_collider: &Collider
    ) -> Self {
        CollisionEndedEvent {
            entity: entity,
            other_position: other_transform.translation.truncate(),
            other_velocity: other_physics.velocity,
            other_mass: other_physics.mass,
            other_radius: other_collider.radius
        }
    }
}

#[derive(Copy, Clone)]
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

fn collision_update_step(
    query: Query<(Entity, &Physics, &Collider, &Transform)>,
    mut cache: ResMut<CollisionCache>,
    start_event_writer: EventWriter<CollisionBeganEvent>,
    update_event_writer: EventWriter<CollisionUpdateEvent>,
    end_event_writer: EventWriter<CollisionEndedEvent>
) {
    broad_phase(&query, &mut cache);
    narrow_phase(&query, &mut cache, start_event_writer, update_event_writer, end_event_writer);
}

// From frame-to-frame, expects the cache to have been cleared
fn broad_phase(
    query: &Query<(Entity, &Physics, &Collider, &Transform)>,
    cache: &mut ResMut<CollisionCache>
) {
    // TODO: Should I be saving this memory?? (This should actually disappear with spacial partitioning)
    let mut seen_pairs = HashSet::new();

    for (entity_a, physics_a, collidable_a, transform_a) in query.iter() {
        for (entity_b, physics_b, collidable_b, transform_b) in query.iter() {
            let pair = CollisionPair {
                entity_a: entity_a,
                entity_b: entity_b
            };

            if entity_a == entity_b || seen_pairs.contains(&pair) {
                continue;
            }

            // TODO: SIMD
            let half_bound_a = collidable_a.bounds.extend(0.0) / 2.0;
            let half_bound_b = collidable_b.bounds.extend(0.0) / 2.0;
            let a_min = transform_a.translation - half_bound_a;
            let a_max = transform_a.translation + half_bound_a;
            let b_min = transform_b.translation - half_bound_b;
            let b_max = transform_b.translation + half_bound_b;

            if a_min.x > b_max.x || a_max.x < b_min.x || a_min.y > b_max.y || a_max.y < b_min.y {
                continue;
            }

            cache.potential_pairs.insert(pair);
            seen_pairs.insert(pair);
        }
    }
}

fn narrow_phase(
    query: &Query<(Entity, &Physics, &Collider, &Transform)>,
    cache: &mut ResMut<CollisionCache>,
    mut start_event_writer: EventWriter<CollisionBeganEvent>,
    mut update_event_writer: EventWriter<CollisionUpdateEvent>,
    mut end_event_writer: EventWriter<CollisionEndedEvent>
) {
    let CollisionCache { potential_pairs, live_pairs } = cache.as_mut();

    live_pairs.retain(|pair| {
        let potentially_colliding = potential_pairs.contains(pair);

        if !potentially_colliding {
            send_collision_event::<CollisionEndedEvent>(query, pair.entity_a, pair.entity_b, &mut end_event_writer);
        }

        potentially_colliding
    });

    for pair in potential_pairs.drain() {
        let (_, physics_a, collidable_a, transform_a) = match query.get(pair.entity_a) {
            Ok(entity) => entity,
            Err(_) => continue
        };

        let (_, physics_b, collidable_b, transform_b) = match query.get(pair.entity_b) {
            Ok(entity) => entity,
            Err(_) => continue
        };

        let are_colliding = transform_a.translation.truncate().distance(transform_b.translation.truncate()) < collidable_a.radius + collidable_b.radius;

        if are_colliding && !live_pairs.contains(&pair) {
            send_collision_event::<CollisionBeganEvent>(query, pair.entity_a, pair.entity_b, &mut start_event_writer);
            live_pairs.insert(pair);
        } else if are_colliding && live_pairs.contains(&pair) {
            send_collision_event::<CollisionUpdateEvent>(query, pair.entity_a, pair.entity_b, &mut update_event_writer);
        } else if !are_colliding && live_pairs.contains(&pair) {
            send_collision_event::<CollisionEndedEvent>(query, pair.entity_a, pair.entity_b, &mut end_event_writer);
            live_pairs.remove(&pair);
        }
    }
}

fn send_collision_event<T>(
    query: &Query<(Entity, &Physics, &Collider, &Transform)>,
    entity_a: Entity,
    entity_b: Entity,
    event_writer: &mut EventWriter<T>
) where T: Event + EventConstructor {
    let (entity_a, physics_a, collider_a, transform_a) = match query.get(entity_a) {
        Ok(entity) => entity,
        Err(_) => return
    };

    let (entity_b, physics_b, collider_b, transform_b) = match query.get(entity_b) {
        Ok(entity) => entity,
        Err(_) => return
    };

    event_writer.send(T::new(entity_a, transform_b, physics_b, collider_b));
    event_writer.send(T::new(entity_b, transform_a, physics_a, collider_a));
}

fn debug_collision_update_step(
    query: Query<(Entity, &Collider, &Transform)>,
    mut start_event_reader: EventReader<CollisionBeganEvent>,
    mut update_event_reader: EventReader<CollisionUpdateEvent>,
    mut end_event_reader: EventReader<CollisionEndedEvent>,
    mut shapes: ResMut<DebugShapes>
) {
    let mut colors = HashMap::new();

    for event in start_event_reader.iter() {
        colors.insert(event.entity, Color::RED);
    }

    for event in update_event_reader.iter() {
        colors.insert(event.entity, Color::YELLOW);
    }

    for event in end_event_reader.iter() {
        colors.insert(event.entity, Color::GREEN);
    }

    for (entity, collidable, transform) in query.iter() {
        let color = colors.get(&entity).unwrap_or(&Color::WHITE);

        shapes.rect()
            .position(transform.translation)
            .size(collidable.bounds)
            .color(color.clone());
    }
}
