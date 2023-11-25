use super::bundles::FixedLifetime;
use super::animation::Animation;
use super::despawn::Despawn;
use bevy::ecs::system::Query;
use bevy::prelude::Res;
use bevy::prelude::Commands;
use bevy::time::{Time, Timer, TimerMode};
use bevy::ecs::entity::Entity;

pub fn projectile_state_update_step(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FixedLifetime, &mut Animation)>,
    time: Res<Time>
) {
    for (entity, mut lifetime, mut animation) in query.iter_mut() {
        lifetime.lifetime.tick(time.delta());

        if lifetime.lifetime.finished() {
            animation.frame_start = 3;
            animation.frame_end = 7;
            animation.frame = 3;
            animation.restart = false;

            commands.entity(entity).insert(Despawn {
                delay: Timer::from_seconds(1.0, TimerMode::Once)
            }).remove::<FixedLifetime>();
        }
    }
}
