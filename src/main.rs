mod game;

use bevy::ecs::event::EventReader;
use bevy::input::mouse::{MouseWheel, MouseScrollUnit};
use bevy::render::camera::OrthographicProjection;
use game::collision::{CircleCollider, CollisionPlugin, CircleColliderBundle, AxisAlignedBoundingBox};
use game::gravity::GravityPlugin;
use game::collision::Physics;
use bevy_prototype_debug_lines::DebugLinesPlugin;

use rand::random;

use bevy::a11y::AccessibilityPlugin;
use bevy::app::{Startup, PluginGroup, PluginGroupBuilder, Update};
use bevy::core::{TaskPoolPlugin, TypeRegistrationPlugin, FrameCountPlugin};
use bevy::core_pipeline::CorePipelinePlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::hierarchy::HierarchyPlugin;
use bevy::input::InputPlugin;
use bevy::log::{LogPlugin, Level};
use bevy::render::texture::ImagePlugin;
use bevy::time::TimePlugin;
use bevy::ecs::bundle::Bundle;
use bevy::asset::{Assets, AssetPlugin};
use bevy::ecs::system::{ResMut, Query};
use bevy::math::{Vec3, Vec2};
use bevy::render::color::Color;
use bevy::render::mesh::Mesh;
use bevy::render::mesh::shape::Circle;
use bevy::sprite::{MaterialMesh2dBundle, Material2d, ColorMaterial, SpritePlugin};
use bevy::transform::components::Transform;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::prelude::{App, Commands};
use bevy::winit::WinitPlugin;
use bevy::window::WindowPlugin;
use bevy::render::RenderPlugin;
use bevy::transform::TransformPlugin;
use bevy::utils::default;

#[derive(Bundle)]
struct Mesh2dColliderBundle<M> where M: Material2d {
    mesh: MaterialMesh2dBundle<M>,
    physics: Physics,
    collider: CircleColliderBundle
}

struct EnginePlugins;

impl PluginGroup for EnginePlugins {
    fn build(self) -> PluginGroupBuilder {
        let group = PluginGroupBuilder::start::<Self>()
            .add(LogPlugin { filter: "warn,collision_engine=debug".into(), level: Level::DEBUG })
            .add(TaskPoolPlugin::default())
            .add(TypeRegistrationPlugin)
            .add(FrameCountPlugin)
            .add(TimePlugin)
            .add(TransformPlugin)
            .add(HierarchyPlugin)
            .add(DiagnosticsPlugin)
            .add(InputPlugin)
            .add(WindowPlugin::default())
            .add(AccessibilityPlugin)
            .add(AssetPlugin::default())
            .add(WinitPlugin::default())
            .add(RenderPlugin::default())
            .add(ImagePlugin::default())
            .add(CorePipelinePlugin)
            .add(SpritePlugin);

        group
    }
}

fn main() {
    App::new()
        .add_systems(Startup, load_startup_elements)
        .add_systems(Update, input_update_step)
        .add_plugins(EnginePlugins)
        .add_plugins(DebugLinesPlugin::default())
        .add_plugins((
            GravityPlugin::default(),
            CollisionPlugin::default()
        ))
        .run();
}

pub fn input_update_step(
    mut query: Query<&mut OrthographicProjection>,
    mut scroll_reader: EventReader<MouseWheel>
) {
    let mut count = 0.0;

    for ev in scroll_reader.iter() {
        match ev.unit {
            MouseScrollUnit::Line => {
                count += ev.y;
            }
            MouseScrollUnit::Pixel => {
                count += ev.y;
            }
        }
    }

    for mut projection in query.iter_mut() {
        projection.scale = (projection.scale + 0.1 * count).clamp(0.10, 10.0);
    }
}

fn load_startup_elements(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    let colors = vec!["#FF8578", "#FFB578", "#FFD86B", "8AFF8A", "#87E5FA", "#C58AFF", "#FC88D4"];
    commands.spawn(Camera2dBundle::default());

    for _ in 0..400 {
        let lo_r = 20.0 * (0.5 + (0.0 / 2.0));
        let hi_r = 20.0 * (0.5 + (1.0 / 2.0));
        let lo_m = 0.15 * lo_r * (0.0 + 1.0);
        let hi_m = 0.15 * hi_r * (1.0 + 1.0);

        let radius = 20.0 * (0.5 + random::<f32>() / 2.0);
        let mass = 0.15 * radius * (random::<f32>() + 1.0);
        let circle = meshes.add(Mesh::from(Circle::new(radius))).into();
        
        let d = (mass - lo_m) / (hi_m - lo_m);
        let color = match Color::hex(colors[(d * colors.len() as f32) as usize]) {
            Ok(color) => color,
            Err(_) => Color::WHITE
        };

        let material = materials.add(ColorMaterial::from(color)).into();
        let pos = Transform::IDENTITY.with_translation(Vec3::new(
            750.0 * (2.0 * random::<f32>() - 1.0),
            750.0 * (2.0 * random::<f32>() - 1.0),
            0.0
        ));

        commands.spawn(Mesh2dColliderBundle {
            mesh: MaterialMesh2dBundle {
                mesh: circle,
                material: material,
                transform: pos,
                ..default()
            },
            physics: Physics::new(
                Vec2::ZERO,
                mass,
                pos
            ),
            collider: CircleColliderBundle {
                aabb: AxisAlignedBoundingBox {
                    bounds: 2.0 * Vec2::new(radius, radius)
                },
                collider: CircleCollider {
                    radius: radius
                }
            }
        });
    }

    /*commands.spawn(StaticBoxColliderBundle {
        aabb: AxisAlignedBoundingBox {
            bounds: Vec2::new(50.0, 1600.0)
        },
        transform: Transform::IDENTITY.with_translation(Vec3::new(-825.0, 0.0, 0.0))
    });

    commands.spawn(StaticBoxColliderBundle {
        aabb: AxisAlignedBoundingBox {
            bounds: Vec2::new(50.0, 1600.0)
        },
        transform: Transform::IDENTITY.with_translation(Vec3::new(825.0, 0.0, 0.0))
    });

    commands.spawn(StaticBoxColliderBundle {
        aabb: AxisAlignedBoundingBox {
            bounds: Vec2::new(1600.0, 50.0)
        },
        transform: Transform::IDENTITY.with_translation(Vec3::new(0.0, -825.0, 0.0))
    });

    commands.spawn(StaticBoxColliderBundle {
        aabb: AxisAlignedBoundingBox {
            bounds: Vec2::new(1600.0, 50.0)
        },
        transform: Transform::IDENTITY.with_translation(Vec3::new(0.0, 825.0, 0.0))
    });*/
}
