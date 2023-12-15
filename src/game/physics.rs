use bevy::app::{App, Plugin, Update};
use bevy::ecs::component::Component;
use bevy::math::Vec2;
use bevy::ecs::system::{Query, ResMut};
use bevy::time::Time;
use bevy::prelude::{Res, Transform};
use bevy::render::color::Color;
use bevy_prototype_debug_lines::DebugLines;

#[derive(Component)]
pub struct Physics {
    pub force: Vec2,
    pub mass: f32,
    pub velocity: Vec2
}

pub struct PhysicsPlugin {
    pub enable_debug: bool
}

// TODO: How do we handle the fact that this depends on another plugin?
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, physics_update_step);
        
        if self.enable_debug {
            app.add_systems(Update, debug_physics_update_step);
        }
    }
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        PhysicsPlugin { enable_debug: false }
    }
}

fn physics_update_step(
    mut query: Query<(&mut Physics, &mut Transform)>,
    time: Res<Time>
) {
    for (mut physics, mut transform) in query.iter_mut() {
        let acceleration = physics.force / physics.mass;
        let dt = time.delta_seconds();
        let dv = acceleration * dt;
        physics.velocity += dv;    
        let dp = physics.velocity * dt;
        transform.translation += dp.extend(0.0);
    }
}

fn debug_physics_update_step(
    query: Query<(&Physics, &Transform)>,
    mut lines: ResMut<DebugLines>
) {
    for (physics, transform) in query.iter() {
        lines.line_colored(
            transform.translation,
            transform.translation + physics.velocity.extend(0.0),
            0.0,
            Color::YELLOW
        );

        lines.line_colored(
            transform.translation,
            transform.translation + physics.force.extend(0.0),
            0.0,
            Color::RED
        );
    }
}
