use super::collision::{CollisionBeganEvent, Collider, CollisionUpdateEvent};
use super::physics::Physics;
use bevy::app::{App, Plugin, Update};
use bevy::ecs::entity::Entity;
use bevy::ecs::event::EventReader;
use bevy::ecs::system::Query;
use bevy::prelude::Transform;

pub struct CollisionResolverPlugin;

impl Plugin for CollisionResolverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            collision_resolver_update_step,
            overlap_resolver_update_step
        ));
    }
}

impl Default for CollisionResolverPlugin {
    fn default() -> Self {
        CollisionResolverPlugin {}
    }
}

fn collision_resolver_update_step(
    mut query: Query<(Entity, &mut Physics, &Transform)>,
    mut start_event_reader: EventReader<CollisionBeganEvent>
) {
    for event in start_event_reader.iter() {
        let (_, mut physics, transform) = match query.get_mut(event.entity) {
            Ok(entity) => entity,
            Err(_) => continue
        };

        let anti_vector = (transform.translation.truncate() - event.other_position).normalize();
        physics.velocity = anti_vector * physics.velocity.length();
    }
}

fn overlap_resolver_update_step(
    mut query: Query<(Entity, &Collider, &mut Transform)>,
    mut update_event_reader: EventReader<CollisionUpdateEvent>
) {
    for event in update_event_reader.iter() {
        let (_, collider, mut transform) = match query.get_mut(event.entity) {
            Ok(entity) => entity,
            Err(_) => continue
        };

        let anti_vector = transform.translation.truncate() - event.other_position;
        let overflow = (collider.radius + event.other_radius - anti_vector.length()).abs().min(1.0);
        transform.translation = transform.translation + (overflow * anti_vector.normalize().extend(0.0));
    }
}
