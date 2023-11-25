use super::animation::Animation;
use super::physics::Physics;
use bevy::ecs::component::Component;
use bevy::ecs::bundle::Bundle;
use bevy::sprite::{TextureAtlas, SpriteSheetBundle};
use bevy::time::Timer;
use bevy::prelude::SpatialBundle;
use bevy::ecs::system::Resource;
use bevy::prelude::{Handle, Image};

#[derive(Resource)]
pub struct GlobalAssets {
    pub spaceship: Handle<Image>,
    pub thruster: Handle<TextureAtlas>,
    pub projectile: Handle<TextureAtlas>
}

#[derive(Component)]
pub struct Player {}

#[derive(Component)]
pub struct FixedLifetime {
    pub lifetime: Timer
}

#[derive(Component)]
pub struct Ship {
    pub linear_power: f32,
    pub angular_power: f32,
    pub reload: Timer
}

#[derive(Bundle)]
pub struct AnimatedSpriteBundle {
    pub animation: Animation,
    pub sprite: SpriteSheetBundle
}

#[derive(Bundle)]
pub struct ProjectileBundle {
    pub sprite: AnimatedSpriteBundle,
    pub physics: Physics,
    pub lifetime: FixedLifetime
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub ship: Ship,
    pub physics: Physics,
    pub position: SpatialBundle
}
