use super::bundles::CollisionEvent;
use super::physics::Physics;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::EventReader;
use bevy::ecs::system::Query;
use bevy::prelude::Transform;

pub fn bounce_update_step(
    mut query: Query<(Entity, &mut Physics, &Transform)>,
    mut event_reader: EventReader<CollisionEvent>
) {
    for event in event_reader.iter() {
        let [(_, mut physics_a, transform_a), (_, _, transform_b)] = match query.get_many_mut([event.entity, event.other]) {
            Ok(entity) => entity,
            Err(_) => continue
        };

        bounce(&mut physics_a, transform_a, transform_b);
    }
}

fn bounce(
    physics: &mut Physics,
    transform: &Transform,
    other: &Transform,
) {
    physics.velocity = physics.velocity.length() * ((transform.translation - other.translation).normalize().truncate());
}
