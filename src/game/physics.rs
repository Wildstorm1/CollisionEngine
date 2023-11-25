use bevy::ecs::component::Component;
use bevy::math::{Vec3, Quat};
use bevy::ecs::system::Query;
use bevy::time::Time;
use bevy::prelude::{Res, Transform};

#[derive(Component)]
pub struct Physics {
    pub acceleration: Vec3,    
    pub velocity: Vec3,
    pub max_speed: Option<f32>,
    pub angular_velocity: f32,
    pub angular_acceleration: f32,
    pub max_angular_speed: Option<f32>,
}

pub fn physics_update_step(
    mut query: Query<(&mut Physics, &mut Transform)>,
    time: Res<Time>
) {
    for (mut physics, mut transform) in query.iter_mut() {
        update_linear_motion(&mut physics, &mut transform, &time);
        update_angular_motion(&mut physics, &mut transform, &time);
    }
}

fn update_linear_motion(
    physics: &mut Physics,
    transform: &mut Transform,
    time: &Res<Time>
) {
    let dv = physics.acceleration * time.delta_seconds();
    physics.velocity += dv;

    physics.velocity = match physics.max_speed {
        Some(limit) => physics.velocity.clamp_length_max(limit),
        None => physics.velocity
    };

    let dp = physics.velocity * time.delta_seconds();
    transform.translation += dp;
}

fn update_angular_motion(
    physics: &mut Physics,
    transform: &mut Transform,
    time: &Res<Time>
) {
    let d1theta = physics.angular_acceleration * time.delta_seconds();
    physics.angular_velocity += d1theta;

    physics.angular_velocity = match physics.max_angular_speed {
        Some(limit) => physics.angular_velocity.clamp(-limit, limit),
        None => physics.angular_velocity
    };

    let dtheta = physics.angular_velocity * time.delta_seconds();
    let (_, theta) = transform.rotation.to_axis_angle();
    transform.rotation = Quat::from_rotation_z(
        f32::rem_euclid(theta + dtheta, 2.0 * std::f32::consts::PI)
    );
}
