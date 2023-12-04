use std::collections::HashSet;
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
    // TODO: I don't know how I feel about having to do this filter.
    // Some systems may care about which specific entity (ie a ship)
    // Others like this hypothetically don't care, but the physics does
    // TODO: Once the velocities are changed, we technically shouldn't change them again
    // this points towards maybe using a force?
    let mut set = HashSet::new();

    for event in event_reader.iter() {
        let [(_, mut physics_a, transform_a), (_, mut physics_b, transform_b)] = match query.get_many_mut([event.entity, event.other]) {
            Ok(entity) => entity,
            Err(_) => continue
        };

        if !set.contains(&event.entity) && !set.contains(&event.other) {
            set.insert(event.entity);
            set.insert(event.other);
            bounce(&mut physics_a, transform_a, &mut physics_b, transform_b);
        }
    }
}

fn bounce(
    physics: &mut Physics,
    transform: &Transform,
    physics_b: &mut Physics,
    transform_b: &Transform,
) {
    physics.velocity = physics.velocity.length() * ((transform.translation - transform_b.translation).normalize().truncate());
    physics_b.velocity = physics_b.velocity.length() * ((transform_b.translation - transform.translation).normalize().truncate());
}
