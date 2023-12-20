use super::physics::Physics;
use bevy::app::{App, Plugin, PreUpdate};
use bevy::ecs::system::Query;
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
    mut query: Query<(&mut Physics, &Transform)>
) {
    for (mut physics, transform) in query.iter_mut() {
        let dist = transform.translation.length().min(50.0);
        physics.force = 0.75 * -dist * transform.translation.truncate().normalize();
    }
}
