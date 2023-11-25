mod game;

use game::animation::{Animation, animation_update_step};
use game::despawn::despawn_update_step;
use game::input::player_input_update_step;
use game::physics::{Physics, physics_update_step};
use game::projectile::projectile_state_update_step;
use game::ships::ship_state_update_step;
use game::bundles::{
    PlayerBundle,
    GlobalAssets,
    AnimatedSpriteBundle,
    Ship,
    Player
};

use std::time::Duration;
use bevy::asset::Assets;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::ResMut;
use bevy::hierarchy::BuildChildren;
use bevy::math::Vec2;
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
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_sprite_assets)
            .add_systems(Startup, load_startup_elements)
            .add_systems(Update, (
                player_input_update_step,
                animation_update_step,
                projectile_state_update_step,
                despawn_update_step,
                physics_update_step.after(player_input_update_step),
                ship_state_update_step.after(player_input_update_step)
            ));
    }
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
