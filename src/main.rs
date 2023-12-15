mod game;
use bevy::app::Startup;
use bevy_prototype_debug_lines::DebugLinesPlugin;
use game::physics::{Physics, PhysicsPlugin};

use bevy::ecs::bundle::Bundle;
use bevy::asset::Assets;
use bevy::ecs::system::ResMut;
use bevy::math::{Vec3, Vec2};
use bevy::render::color::Color;
use bevy::render::mesh::Mesh;
use bevy::render::mesh::shape::Circle;
use bevy::sprite::{MaterialMesh2dBundle, Material2d, ColorMaterial};
use bevy::transform::components::Transform;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::prelude::{App, Commands};
use bevy::DefaultPlugins;
use bevy::utils::default;
use rand::random;

#[derive(Bundle)]
struct Mesh2dColliderBundle<M> where M: Material2d {
    mesh: MaterialMesh2dBundle<M>,
    physics: Physics
}

fn main() {
    App::new()
        .add_systems(Startup, load_startup_elements)
        .add_plugins((
            DefaultPlugins,
            DebugLinesPlugin::default(),
            PhysicsPlugin { enable_debug: true }
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
                    300.0 * (2.0 * random::<f32>() - 1.0),
                    0.0
                )),
                ..default()
            },
            physics: Physics {
                force: 50.0 * Vec2::new(
                    2.0 * (random::<f32>() - 0.5),
                    2.0 * (random::<f32>() - 0.5)
                ),
                mass: 0.15 * radius * (random::<f32>() + 1.0),
                velocity: Vec2::ZERO
            }
        });
    }
}
