use super::plugin::Physics;
use bevy::app::{App, Plugin, PreUpdate};
use bevy::ecs::system::{Query, Res};
use bevy::math::Vec2;
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
    for (mut physics, _) in query.iter_mut() {
        physics.force = Vec2::new(0.0, -1500.0);
    }
}