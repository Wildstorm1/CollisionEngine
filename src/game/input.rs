use super::bundles::GlobalAssets;
use super::bundles::Player;
use super::bundles::Ship;
use super::bundles::ProjectileBundle;
use super::bundles::AnimatedSpriteBundle;
use super::bundles::FixedLifetime;
use super::physics::Physics;
use super::animation::Animation;
use std::time::Duration;
use bevy::ecs::system::Query;
use bevy::ecs::query::With;
use bevy::prelude::{Transform, Res, Vec3};
use bevy::sprite::TextureAtlasSprite;
use bevy::math::Quat;
use bevy::prelude::Commands;
use bevy::time::{Stopwatch, Timer, TimerMode};
use bevy::input::Input;
use bevy::input::keyboard::KeyCode;
use bevy::sprite::Anchor;
use bevy::sprite::SpriteSheetBundle;
use bevy::utils::default;

pub fn player_input_update_step(
    mut commands: Commands,
    mut query: Query<(&mut Ship, &mut Physics, &Transform), With<Player>>,
    keys: Res<Input<KeyCode>>,
    globals: Res<GlobalAssets>
) {
    // TODO: Think about how we could plug a different driver into this (ie, ai)
    // It may be as simple as swapping the Player tag for an AI tag on the entity
    let (mut ship, mut physics, transform) = query.single_mut();

    if keys.pressed(KeyCode::A) {
        ship.look_angle = f32::rem_euclid(
            ship.look_angle + ship.angular_power, 2.0 * std::f32::consts::PI
        );
    }

    if keys.pressed(KeyCode::D) {
        ship.look_angle = f32::rem_euclid(
            ship.look_angle - ship.angular_power, 2.0 * std::f32::consts::PI
        );
    }

    let look_vector = Quat::from_rotation_z(ship.look_angle)
        .mul_vec3(Vec3::new(0.0, 1.0, 0.0))
        .normalize();

    if keys.pressed(KeyCode::W) {
        physics.acceleration = look_vector * ship.linear_power;
    } else {
        physics.acceleration = Vec3::new(0.0, 0.0, 0.0);
    }

    if keys.pressed(KeyCode::Space) && ship.reload.finished() {
        ship.reload.reset();

        let mut bullet_sprite = TextureAtlasSprite::new(0);
        bullet_sprite.anchor = Anchor::BottomCenter;

        let mut c_frame = transform.clone();
        c_frame.translation += 4.0 * look_vector;
        c_frame.translation.z = 0.0;

        // TODO: Technically, we can still use a factory :)
        // Even better, that factory can be in the system, which keeps fields private
        commands.spawn(ProjectileBundle {
            sprite: AnimatedSpriteBundle {
                animation: Animation {
                    time: Stopwatch::new(),
                    period: Duration::from_millis(100),
                    frame_start: 0,
                    frame_end: 2,
                    frame: 0,
                    restart: true
                },
                sprite: SpriteSheetBundle {
                    sprite: bullet_sprite,
                    texture_atlas: globals.projectile.clone(),
                    transform: c_frame,
                    ..default()
                }
            },
            physics: Physics {
                velocity: physics.velocity + (35.0 * look_vector),
                acceleration: Vec3::new(0.0, 0.0, 0.0),
                max_speed: None
            },
            lifetime: FixedLifetime {
                lifetime: Timer::from_seconds(3.0, TimerMode::Once)
            }
        });
    }
}
