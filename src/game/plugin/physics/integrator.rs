use bevy::ecs::component::Component;
use bevy::ecs::system::{Query, Resource, ResMut};
use bevy::math::Vec2;
use bevy::transform::components::Transform;

#[derive(Resource)]
pub(in super::super) struct Simulation {
    iterations: usize,
    time_step_seconds: f32
}

impl Simulation {
    pub(in super::super) fn new(iters: usize) -> Self {
        Self {
            iterations: iters,
            time_step_seconds: 0.0
        }
    }

    pub(in super::super) fn set_time(&mut self, time: f32) {
        self.time_step_seconds = time / self.iterations as f32;
    }

    pub(in super::super) fn iterations(&self) -> usize {
        self.iterations
    }
}

#[derive(Component)]
pub struct Physics {
    pub force: Vec2,
    pub mass: f32,
    pub(in super::super) prev_pos: Vec2
}

impl Physics {
    pub fn new(force: Vec2, mass: f32, position: Transform) -> Self {
        Self {
            force: force,
            mass: mass,
            prev_pos: position.translation.truncate()
        }
    }

    pub fn with_velocity(force: Vec2, mass: f32, position: Transform, velocity: Vec2) -> Self {
        Self {
            force: force,
            mass: mass,
            prev_pos: position.translation.truncate() - velocity
        }
    }
}

pub(in super::super) fn integration_phase(
    mut query: Query<(&mut Physics, &mut Transform)>,
    cache: ResMut<Simulation>
) {
    let dt = cache.time_step_seconds;

    for (mut physics, mut transform) in query.iter_mut() {
        let cur_pos = transform.translation;
        let acceleration = physics.force / physics.mass;
        transform.translation = 2.0 * transform.translation - physics.prev_pos.extend(0.0) + acceleration.extend(0.0) * dt * dt;
        physics.prev_pos = cur_pos.truncate();
    }
}