mod game;

use bevy_prototype_debug_lines::DebugLinesPlugin;
use game::animation::{Animation, animation_update_step};
use game::debug_boxs::debug_boxs_update_step;
use game::despawn::despawn_update_step;
use game::input::player_input_update_step;
use game::physics::{Physics, physics_update_step};
use game::projectile::projectile_state_update_step;
use game::ships::ship_state_update_step;
use game::collision::{Collidable, collision_update_step};
use game::bundles::{
    AsteroidBundle,
    PlayerBundle,
    GlobalAssets,
    AnimatedSpriteBundle,
    Ship,
    Player,
    CollisionEvent
};

use std::time::Duration;
use rand::prelude::random;
use bevy::asset::Assets;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::ResMut;
use bevy::hierarchy::BuildChildren;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{
    App,
    Res,
    Plugin,
    Commands,
    Handle,
    AssetServer,
    Image,
    Transform, SpatialBundle
};
use bevy::DefaultPlugins;
use bevy::app::{
    PreStartup,
    Startup,
    Update
};
use bevy::sprite::{
    TextureAtlas,
    SpriteBundle,
    SpriteSheetBundle,
    TextureAtlasSprite,
    Anchor
};
use bevy::time::{
    Stopwatch,
    Timer,
    TimerMode
};
use bevy::utils::default;

struct MainPlugin;

impl Plugin for MainPlugin {
    // TODO: Refactor the collision and physics systems to be plugins
    fn build(&self, app: &mut App) {
        app.add_event::<CollisionEvent>()
            .add_systems(PreStartup, load_sprite_assets)
            .add_systems(Startup, load_startup_elements)
            .add_systems(Update, (
                player_input_update_step,
                animation_update_step,
                projectile_state_update_step,
                despawn_update_step,
                physics_update_step.after(player_input_update_step),
                ship_state_update_step.after(player_input_update_step),
                collision_update_step.after(physics_update_step),
                debug_boxs_update_step.after(collision_update_step)
            ));
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MainPlugin,
            DebugLinesPlugin::default()
        )).run();
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

    let raw_ast_atlas: Handle<Image> = asset_server.load("../assets/Asteroid.png");
    let ast_atlas = TextureAtlas::from_grid(
        raw_ast_atlas,
        Vec2::new(32.0, 32.0),
        8,
        1,
        None,
        None
    );

    commands.insert_resource(GlobalAssets{
        spaceship: asset_server.load("../assets/Ship.png"),
        thruster: atlas.add(thruster_atlas),
        projectile: atlas.add(proj_atlas),
        asteroid: atlas.add(ast_atlas)
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
            angular_power: 2.0,
            reload: Timer::from_seconds(0.4, TimerMode::Once)
        },
        physics: Physics {
            force: Vec2::ZERO,
            mass: 1.0,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            max_speed: Some(75.0),
            angular_velocity: 0.0
        },
        position: SpatialBundle {
            ..default()
        },
        collidable: Collidable {
            bounds: Vec2::new(18.0, 18.0)
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

    let asteroid_sprite = TextureAtlasSprite::new(0);

    for _ in 0..30 {
        let mut dx = 50.0 * random::<f32>() + 0.5;
        let mut dy = 50.0 * random::<f32>() + 0.5;
        let scale = Vec2::new(
            2.0 * random::<f32>() + 1.0,
            2.0 * random::<f32>() + 1.0
        );

        let bound_r = (6.0 * scale).length();

        if random::<bool>() {
            dx *= -1.0;
        }

        if random::<bool>() {
            dy *= -1.0;
        }

        commands.spawn(AsteroidBundle {
            sprite: SpriteSheetBundle {
                sprite: asteroid_sprite.clone(),
                texture_atlas: globals.asteroid.clone(),
                transform: Transform::IDENTITY.with_translation(Vec3::new(
                    200.0 * (2.0 * random::<f32>() - 1.0),
                    200.0 * (2.0 * random::<f32>() - 1.0),
                    0.0
                )).with_scale(Vec3::new(
                    scale.x,
                    scale.y,
                    0.0
                )),
                ..default()
            },
            physics: Physics {
                force: Vec2::ZERO,
                mass: 1.0,
                velocity: Vec2::new(dx, dy),
                acceleration: Vec2::ZERO,
                max_speed: None,
                angular_velocity: 3.5 * (2.0 * random::<f32>() - 1.0)
            },
            collidable: Collidable {
                bounds: Vec2::new(2.0 * bound_r, 2.0 * bound_r)
            }
        });
    }

    commands.entity(player_id).push_children(&[ship_id, exhaust_id]);
}
