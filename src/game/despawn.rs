use bevy::ecs::entity::Entity;
use bevy::prelude::Commands;
use bevy::ecs::component::Component;
use bevy::time::{Time, Timer};
use bevy::ecs::system::Query;
use bevy::prelude::Res;

#[derive(Component)]
pub struct Despawn {
    pub delay: Timer
}

pub fn despawn_update_step(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Despawn)>,
    time: Res<Time>
) {
    for (entity, mut despawn) in query.iter_mut() {
        despawn.delay.tick(time.delta());

        if despawn.delay.finished() {
            commands.entity(entity).remove::<Despawn>().despawn();
        }
    }
}
