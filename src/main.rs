use std::time::Duration;

use bevy::asset::Assets;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{
    Resource,
    Query,
    ResMut
};
use bevy::hierarchy::{BuildChildren, Children};
use bevy::input::Input;
use bevy::input::keyboard::KeyCode;
use bevy::math::{
    Quat,
    Vec2
};
use bevy::prelude::{
    App,
    Res,
    Plugin,
    Commands,
    Handle,
    AssetServer,
    Image,
    Vec3,
    Transform, SpatialBundle
};
use bevy::ecs::bundle::Bundle;
use bevy::DefaultPlugins;
use bevy::app::{
    PreStartup,
    Startup,
    Update
};
use bevy::render::view::Visibility;
use bevy::sprite::{
    TextureAtlas,
    SpriteBundle,
    SpriteSheetBundle,
    TextureAtlasSprite,
    Anchor
};
use bevy::time::{
    Time,
    Stopwatch, Timer, TimerMode
};
use bevy::utils::default;

struct MainPlugin;

impl Plugin for MainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_sprite_assets)
            .add_systems(Startup, load_startup_elements)
            .add_systems(Update, (
                handle_player_input,
                update_animations,
                update_projectiles,
                despawn_update,
                physics_update.after(handle_player_input),
                update_ships.after(handle_player_input)
            ));
    }
}

#[derive(Resource)]
struct GlobalAssets {
    spaceship: Handle<Image>,
    thruster: Handle<TextureAtlas>,
    projectile: Handle<TextureAtlas>
}

// Components should be reasonably reusable and orthogonal

#[derive(Component)]
struct Player {}

#[derive(Component)]
struct FixedLifetime {
    lifetime: Timer
}

#[derive(Component)]
struct Despawn {
    delay: Timer
}

#[derive(Component)]
struct Ship {
    linear_power: f32,
    angular_power: f32,
    look_angle: f32,
    reload: Timer
}

#[derive(Component)]
struct Physics {
    velocity: Vec3,
    acceleration: Vec3,
    max_speed: Option<f32>
}

// TODO: Should we have different components for different kinds of animations?
// Should different animations have different systems to drive them, ie couple
// the animation kind to the system? Or could we employ a trait and polymorphism
// with different logic? Does this couple things too much? Is it undoing the benefit
// of a composition model?
#[derive(Component)]
struct Animation {
    time: Stopwatch,
    period: Duration,
    frame_start: usize,
    frame_end: usize,
    frame: usize,
    restart: bool
}

// Higher level bundles are not as reusable though some potions may be refactored out

#[derive(Bundle)]
struct AnimatedSpriteBundle {
    animation: Animation,
    sprite: SpriteSheetBundle
}

#[derive(Bundle)]
struct ProjectileBundle {
    sprite: AnimatedSpriteBundle,
    physics: Physics,
    lifetime: FixedLifetime
}

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    ship: Ship,
    physics: Physics,
    position: SpatialBundle
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MainPlugin))
        .run();
}

// TODO: We can make this a Plugin
fn load_sprite_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas: ResMut<Assets<TextureAtlas>>
) {
    let raw_thruster_atlas: Handle<Image> = asset_server.load("../assets/Thruster.png");
    let thruster_atlas = TextureAtlas::from_grid(
        raw_thruster_atlas,
        Vec2::new(8.0, 8.0),
        5,
        1,
        None,
        None
    );

    let raw_proj_atlas: Handle<Image> = asset_server.load("../assets/Projectile.png");
    let proj_atlas = TextureAtlas::from_grid(
        raw_proj_atlas,
        Vec2::new(5.0, 5.0),
        8,
        1,
        None,
        None
    );

    commands.insert_resource(GlobalAssets{
        spaceship: asset_server.load("../assets/Ship.png"),
        thruster: atlas.add(thruster_atlas),
        projectile: atlas.add(proj_atlas)
    });
}

fn load_startup_elements(
    mut commands: Commands,
    globals: Res<GlobalAssets>
) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale *= 0.40;
    commands.spawn(camera);

    // This 'Complex' entity is built from a hierarchy of bundles
    // We do this so that the sprites will translate with the parent,
    // but retains a decoupled state
    let player_id = commands.spawn(PlayerBundle {
        player: Player {},
        ship: Ship {
            linear_power: 50.0,
            angular_power: 0.025,
            look_angle: 0.0,
            reload: Timer::from_seconds(0.4, TimerMode::Once)
        },
        physics: Physics {
            velocity: Vec3::new(0.0, 0.0, 0.0),
            acceleration: Vec3::new(0.0, 0.0, 0.0),
            max_speed: Some(75.0)            
        },
        position: SpatialBundle {
            ..default()
        },
    }).id();

    let ship_id = commands.spawn(SpriteBundle {
        transform: Transform::from_xyz(0.0, 0.0, 1.0),
        texture: globals.spaceship.clone(),
        ..default()
    }).id();

    let mut fire_sprite = TextureAtlasSprite::new(0);
    fire_sprite.anchor = Anchor::TopCenter;

    let exhaust_id = commands.spawn(AnimatedSpriteBundle {
        animation: Animation {
            time: Stopwatch::new(),
            period: Duration::from_millis(100),
            frame_start: 0,
            frame_end: 4,
            frame: 0,
            restart: true
        },
        sprite: SpriteSheetBundle {
            sprite: fire_sprite,
            texture_atlas: globals.thruster.clone(),
            transform: Transform::from_xyz(0.0, -6.0, 0.0),
            ..default()
        },
    }).id();

    commands.entity(player_id).push_children(&[ship_id, exhaust_id]);
}

fn handle_player_input(
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
                velocity: (physics.velocity.length() + 25.0) * look_vector,
                acceleration: Vec3::new(0.0, 0.0, 0.0),
                max_speed: None
            },
            lifetime: FixedLifetime {
                lifetime: Timer::from_seconds(3.0, TimerMode::Once)
            }
        });
    }
}

fn update_animations(
    mut query: Query<(&mut TextureAtlasSprite, &mut Animation)>,
    time: Res<Time>
) {
    for (mut sprite, mut animation) in query.iter_mut() {
        animation.time.tick(time.delta());

        if animation.time.elapsed_secs_f64() > animation.period.as_secs_f64() {
            animation.time.reset();

            if animation.frame < animation.frame_end {
                animation.frame += 1;
            } else if animation.restart {
                animation.frame = animation.frame_start;
            }

            sprite.index = animation.frame;
        }        
    }
}

fn update_ships(
    mut query: Query<(&mut Ship, &Physics, &mut Transform, &Children)>,
    mut sprites: Query<&mut Visibility, With<TextureAtlasSprite>>,
    time: Res<Time>
) {
    for (mut ship, physics, mut transform, children) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_z(ship.look_angle);
        ship.reload.tick(time.delta());

        // A ship will have a small number of children, so we loop over them and look them up in
        // the more general query of all exhaust sprites.
        // We could make the sprites query more specific by tagging exhaust with a tag
        // Also, if no children match the query, this section does nothing
        for &child in children.iter() {
            match sprites.get_mut(child) {
                Ok(mut sprite) => {
                    if physics.acceleration.length() < 3.0 * f32::EPSILON {
                        *sprite = Visibility::Hidden;
                    } else {
                        *sprite = Visibility::Visible;
                    }
                },
                Err(_) => {}
            }
        }
    }
}

fn update_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FixedLifetime, &mut Animation)>,
    time: Res<Time>
) {
    for (entity, mut lifetime, mut animation) in query.iter_mut() {
        lifetime.lifetime.tick(time.delta());

        if lifetime.lifetime.finished() {
            animation.frame_start = 3;
            animation.frame_end = 7;
            animation.frame = 3;
            animation.restart = false;

            commands.entity(entity).insert(Despawn {
                delay: Timer::from_seconds(1.0, TimerMode::Once)
            }).remove::<FixedLifetime>();
        }
    }
}

fn despawn_update(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Despawn)>,
    time: Res<Time>
) {
    for (entity, mut despawn) in query.iter_mut() {
        despawn.delay.tick(time.delta());

        if despawn.delay.finished() {
            commands.entity(entity).remove::<Despawn>().despawn();
        }
    }
}

fn physics_update(
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
