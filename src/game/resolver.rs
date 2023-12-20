use super::collision::{CollisionBeganEvent, Collider, CollisionUpdateEvent};
use super::physics::Physics;
use bevy::app::{App, Plugin, Update};
use bevy::ecs::entity::Entity;
use bevy::ecs::event::EventReader;
use bevy::ecs::system::Query;
use bevy::math::Vec3;
use bevy::prelude::Transform;
use rand::random;

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
    mut query: Query<(Entity, &mut Physics, &mut Transform)>,
    mut start_event_reader: EventReader<CollisionBeganEvent>
) {
    for event in start_event_reader.iter() {
        let (_, mut physics, mut transform) = match query.get_mut(event.entity) {
            Ok(entity) => entity,
            Err(_) => continue
        };

        let anti_vector = transform.translation.truncate() - event.other_position;

        if anti_vector.length_squared() <= f32::EPSILON {
            transform.translation = transform.translation + 10.0 * f32::EPSILON * Vec3::new(2.0 * random::<f32>() - 1.0, 2.0 * random::<f32>() - 1.0, 0.0);
            continue;
        }

        let deflection_n = (2.0 * event.other_mass) * (physics.velocity - event.other_velocity).dot(anti_vector);
        let deflection_d = anti_vector.length_squared() * (physics.mass + event.other_mass);
        physics.velocity = 0.90 * physics.velocity - (deflection_n / deflection_d * anti_vector);
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

        if anti_vector.length_squared() <= f32::EPSILON {
            transform.translation = transform.translation + 10.0 * f32::EPSILON * Vec3::new(2.0 * random::<f32>() - 1.0, 2.0 * random::<f32>() - 1.0, 0.0);
            continue;
        }

        let overflow = (collider.radius + event.other_radius - anti_vector.length()).abs().min(1.0);
        transform.translation = transform.translation + (overflow * anti_vector.normalize().extend(0.0));
    }
}
