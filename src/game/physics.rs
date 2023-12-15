use bevy::ecs::component::Component;
use bevy::math::Vec2;
use bevy::ecs::system::Query;
use bevy::time::Time;
use bevy::prelude::{Res, Transform};

#[derive(Component)]
pub struct Physics {
    pub force: Vec2,
    pub mass: f32,
    pub velocity: Vec2
}

pub fn physics_update_step(
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
