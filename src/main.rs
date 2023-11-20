use std::time::Duration;

use bevy::asset::Assets;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::component::Component;
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
    Stopwatch
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
                physics_update.after(handle_player_input),
                update_ships.after(handle_player_input)
            ));
    }
}

#[derive(Resource)]
struct GlobalAssets {
    spaceship: Handle<Image>,
    thruster: Handle<TextureAtlas>
}

// Components should be reasonably reusable and orthogonal

#[derive(Component)]
struct Player {}

#[derive(Component)]
struct Ship {
    linear_power: f32,
    angular_power: f32,
    look_angle: f32
}

#[derive(Component)]
struct Physics {
    velocity: Vec3,
    acceleration: Vec3,
    max_speed: Option<f32>
}

#[derive(Component)]
struct Animation {
    time: Stopwatch,
    period: Duration,
    frame: usize,
    frames: usize
}

// Higher level bundles are not as reusable though some potions may be refactored out

#[derive(Bundle)]
struct AnimatedSpriteBundle {
    animation: Animation,
    sprite: SpriteSheetBundle
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
    let raw_atlas: Handle<Image> = asset_server.load("../assets/Thruster.png");
    let fire_atlas = TextureAtlas::from_grid(
        raw_atlas,
        Vec2::new(8.0, 8.0),
        5,
        1,
        None,
        None
    );

    commands.insert_resource(GlobalAssets{
        spaceship: asset_server.load("../assets/Gray1.png"),
        thruster: atlas.add(fire_atlas)
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
            look_angle: 0.0
        },
        physics: Physics {
            velocity: Vec3::new(0.0, 0.0, 0.0),
            acceleration: Vec3::new(0.0, 0.0, 0.0),
            max_speed: Some(75.0)            
        },
        position: SpatialBundle {
            ..default()
        }
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
            frame: 0,
            frames: 5
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
    mut query: Query<(&mut Ship, &mut Physics), With<Player>>,
    keys: Res<Input<KeyCode>>
) {
    // TODO: Think about how we could plug a different driver into this (ie, ai)
    // It may be as simple as swapping the Player tag for an AI tag on the entity
    let (mut ship, mut physics) = query.single_mut();

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

    let a = Quat::from_rotation_z(ship.look_angle)
        .mul_vec3(Vec3::new(0.0, 1.0, 0.0))
        .normalize();

    if keys.pressed(KeyCode::W) {
        physics.acceleration = a * ship.linear_power;
    } else {
        physics.acceleration = Vec3::new(0.0, 0.0, 0.0);
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
            animation.frame = (animation.frame + 1) % animation.frames;
            sprite.index = animation.frame;
        }
    }
}

fn update_ships(
    mut query: Query<(&Ship, &Physics, &mut Transform, &Children)>,
    mut sprites: Query<&mut Visibility, With<TextureAtlasSprite>>
) {
    for (ship, physics, mut transform, children) in query.iter_mut() {
        transform.rotation = Quat::from_rotation_z(ship.look_angle);

        for &child in children.iter() {
            match sprites.get_mut(child) {
                Ok(mut sprite) => {
                    if physics.acceleration.length() < 3.0 * f32::EPSILON {
                        *sprite = Visibility::Hidden;
                    } else {
                        *sprite = Visibility::Visible;
                    }
                },
                Err(_) => {
                    println!("Error getting sprite");
                }
            }
        }
    }
}

fn physics_update(
    mut query: Query<(&mut Physics, &mut Transform)>,
    time: Res<Time>
) {
    // Note: Don't couple the look vector here. Look vector should inform input
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
