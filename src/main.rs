mod game;

use game::bundles::Mesh2dColliderBundle;

use bevy::asset::Assets;
use bevy::ecs::system::ResMut;
use bevy::math::Vec3;
use bevy::render::color::Color;
use bevy::render::mesh::Mesh;
use bevy::render::mesh::shape::Circle;
use bevy::sprite::{MaterialMesh2dBundle, ColorMaterial};
use bevy::transform::components::Transform;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::prelude::{App, Plugin, Commands};
use bevy::DefaultPlugins;
use bevy::app::Startup;
use bevy::utils::default;
use rand::random;

struct MainPlugin;

impl Plugin for MainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_startup_elements);
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MainPlugin
        )).run();
}

fn load_startup_elements(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    let colors = vec!["#FFD86B", "#87E5FA", "#FC88D4"];
    commands.spawn(Camera2dBundle::default());

    for _ in 0..100 {
        let radius = 50.0 * (0.5 + random::<f32>() / 2.0);
        let circle = meshes.add(Mesh::from(Circle::new(radius))).into();
        let color = match Color::hex(colors[random::<usize>() % colors.len()]) {
            Ok(color) => color,
            Err(_) => Color::WHITE
        };

        let material = materials.add(ColorMaterial::from(color)).into();

        commands.spawn(Mesh2dColliderBundle {
            mesh: MaterialMesh2dBundle {
                mesh: circle,
                material: material,
                transform: Transform::IDENTITY.with_translation(Vec3::new(
                    500.0 * (2.0 * random::<f32>() - 1.0),
                    500.0 * (2.0 * random::<f32>() - 1.0),
                    0.0
                )),
                ..default()
            }
        });
    }
}
