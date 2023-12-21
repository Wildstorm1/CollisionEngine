use super::collision::Physics;
use bevy::app::{App, Plugin, PreUpdate};
use bevy::ecs::system::{Query, Res};
use bevy::time::Time;
use bevy::transform::components::Transform;

pub struct GravityPlugin;

impl Plugin for GravityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, gravity_update_step);
    }
}

impl Default for GravityPlugin {
    fn default() -> Self {
        GravityPlugin {}
    }
}

fn gravity_update_step(
    mut query: Query<(&mut Physics, &Transform)>,
    time: Res<Time>
) {
    let direction = 50.0 - (time.elapsed_seconds() % 60.0);
    for (mut physics, transform) in query.iter_mut() {
        physics.force = -10.0 * direction * transform.translation.truncate().normalize();
    }
}
