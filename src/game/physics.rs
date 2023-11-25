use bevy::ecs::component::Component;
use bevy::math::Vec3;
use bevy::ecs::system::Query;
use bevy::time::Time;
use bevy::prelude::{Res, Transform};

#[derive(Component)]
pub struct Physics {
    pub acceleration: Vec3,    
    pub velocity: Vec3,
    pub max_speed: Option<f32>
}

pub fn physics_update_step(
    mut query: Query<(&mut Physics, &mut Transform)>,
    time: Res<Time>
) {
    // Note: Don't couple the look vector here. Look vector should inform physics inputs
    // earlier than this is called
    for (mut physics, mut transform) in query.iter_mut() {
        let dv = physics.acceleration * time.delta_seconds();
        physics.velocity += dv;

        physics.velocity = match physics.max_speed {
            Some(limit) => physics.velocity.clamp_length_max(limit),
            None => physics.velocity
        };

        let dp = physics.velocity * time.delta_seconds();
        transform.translation += dp;
    }
}
