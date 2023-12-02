use super::bundles::GlobalAssets;
use super::bundles::Player;
use super::bundles::Ship;
use super::bundles::ProjectileBundle;
use super::bundles::AnimatedSpriteBundle;
use super::bundles::FixedLifetime;
use super::collision::Collidable;
use super::physics::Physics;
use super::animation::Animation;
use std::time::Duration;
use bevy::ecs::system::Query;
use bevy::ecs::query::With;
use bevy::math::{Vec3, Vec2};
use bevy::prelude::{Transform, Res};
use bevy::sprite::TextureAtlasSprite;
use bevy::prelude::Commands;
use bevy::time::{Stopwatch, Timer, TimerMode};
use bevy::input::Input;
use bevy::input::keyboard::KeyCode;
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
    physics.angular_velocity = 0.0;
    physics.force = Vec2::ZERO;

    if keys.pressed(KeyCode::A) {
        physics.angular_velocity = ship.angular_power;
    }

    if keys.pressed(KeyCode::D) {
        physics.angular_velocity = -ship.angular_power;
    }

    let look_vector = transform.rotation.mul_vec3(Vec3::Y).truncate().normalize();

    if keys.pressed(KeyCode::W) {
        physics.force = look_vector * ship.linear_power;
    }

    if keys.pressed(KeyCode::Space) && ship.reload.finished() {
        ship.reload.reset();

        let mut c_frame = transform.clone();
        c_frame.translation = (c_frame.translation.truncate() + 6.0 * look_vector).extend(0.0);

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
                    sprite: TextureAtlasSprite::new(0),
                    texture_atlas: globals.projectile.clone(),
                    transform: c_frame,
                    ..default()
                }
            },
            physics: Physics {
                force: Vec2::ZERO,
                mass: 1.0,
                velocity: physics.velocity + (35.0 * look_vector),
                acceleration: Vec2::ZERO,
                max_speed: None,
                angular_velocity: 0.0
            },
            lifetime: FixedLifetime {
                lifetime: Timer::from_seconds(3.0, TimerMode::Once)
            },
            collidable: Collidable {
                bounds: Vec2::new(5.0, 5.0)
            }
        });
    }
}
