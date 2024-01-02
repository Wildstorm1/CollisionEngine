// TODO: Should I be saving memory?? (Might actually disappear with spacial partitioning?)
// From frame-to-frame, expects the cache to have been cleared
// How to rerun some of these systems multiple times?
// Running the whole system over all memory is a waste with few overlaps, but random access blasts cache
// Obviously, we should try to run some benchmarks
// Part of the issue is that if an object has nowhere to go, then it will constantly fight with its
// neighbors for space. If they cannot move to accommodate, then no space will open to stop the
// jittering. Having velocities causes initial displacement, which then prompts the fighting.
// TODO: I should consider handling objects going outside the grid safely (ie, ignore them or something)
// TODO: Would be better to detect if the cell changed than reconstruct each time
// Should maybe not count the collision as a change in position?
// Verlet fails at high speeds
// Solved jitter problem with multiple passes and a method that doesn't use first class velocity
// TODO: We can keep an inner representation inside the grid, the grid only promises potentially colliding
// TODO: Benchmark the inner representations
// TODO: Write docs that make it clear what the promise is
// TODO: If penetration depth is larger than max, then cap the rebound position length at a max
// TODO: Experiment with reducing data size in grid
// TODO: Experiment with grid representation
// TODO: Experiment with internal grid data representation
// TODO: Add copyright
// TODO: Think about lifetimes (in this case, I think a lot of data is being copied)
// TODO: Consider just returning all the items, we can let external code check for overlaps and store whatever it needs
// This can be done with the filter iterator
// TODO: Would be nice to have tests
// TODO: Maybe rethink the grid iterator to be a bit more generic (ie, more than just 9 cells)
// TODO: Should remove things from grid that are no longer in the query... Ie, if objects despawn
// TODO: Is there an issue with contact starting/ending with subsets?
// You might worry about the duplicated entities in memory, after all, we are effectively
// storing them in multiple places, which is duplicating the data. The only way to 
// reduce this is to instead store them somewhere, and take references to them everywhere else.
// The issue with this is that it will reduce the cache effectiveness of the data structures, since we
// now are jumping around in memory when iterating the array (the references are not guaranteed to be in
// the order we want everywhere). This is a tradeoff between memory and speed, we can't have both.
// You could think about sorting the entity references, and this would actually help, however we then have
// the issue of needing to keep the data structures sorted.
// TODO: I don't really understand the pub/mod system
// TODO: The bounds collision should be generated in the broad or narrow phase, the current is a hack
// TODO: Use functions so fields can remain as private as possible. This might not be idomatic rust, but usually
// hiding fields behind functions ensures internal invariants are maintained

mod game;

use bevy::core_pipeline::clear_color::ClearColor;
use game::plugin::{ResolverMode, BroadPhaseMode, Physics, CircleCollider, CollisionPlugin, CircleColliderBundle, AxisAlignedBoundingBox};
use game::gravity::GravityPlugin;

use bevy_prototype_debug_lines::DebugLinesPlugin;
use rand::random;

use bevy::ecs::event::EventReader;
use bevy::input::mouse::{MouseWheel, MouseScrollUnit};
use bevy::render::camera::OrthographicProjection;
use bevy::a11y::AccessibilityPlugin;
use bevy::app::{Startup, PluginGroup, PluginGroupBuilder, Update};
use bevy::core::{TaskPoolPlugin, TypeRegistrationPlugin, FrameCountPlugin};
use bevy::core_pipeline::CorePipelinePlugin;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::hierarchy::HierarchyPlugin;
use bevy::input::InputPlugin;
use bevy::log::{LogPlugin, Level};
use bevy::render::texture::ImagePlugin;
use bevy::time::{TimePlugin, Timer, TimerMode, Time};
use bevy::ecs::bundle::Bundle;
use bevy::asset::{Assets, AssetPlugin};
use bevy::ecs::system::{ResMut, Query, Resource, Res};
use bevy::math::{Vec3, Vec2};
use bevy::render::color::Color;
use bevy::render::mesh::Mesh;
use bevy::render::mesh::shape::Circle;
use bevy::sprite::{MaterialMesh2dBundle, Material2d, ColorMaterial, SpritePlugin};
use bevy::transform::components::Transform;
use bevy::core_pipeline::core_2d::Camera2dBundle;
use bevy::prelude::{App, Commands};
use bevy::winit::WinitPlugin;
use bevy::window::{WindowPlugin, Window, WindowResolution};
use bevy::render::RenderPlugin;
use bevy::transform::TransformPlugin;
use bevy::utils::default;

#[derive(Resource)]
struct Objects {
    count: usize,
    timer: Timer
}

#[derive(Bundle)]
struct Mesh2dColliderBundle<M> where M: Material2d {
    mesh: MaterialMesh2dBundle<M>,
    physics: Physics,
    collider: CircleColliderBundle
}

struct EnginePlugins {
    canvas: Option<String>,
    canvas_size: Option<(f32, f32)>
}

impl Default for EnginePlugins {
    fn default() -> Self {
        Self {
            canvas: None,
            canvas_size: None
        }
    }
}

impl EnginePlugins {
    pub fn new(canvas: String, canvas_size: (f32, f32)) -> Self {
        Self {
            canvas: Some(canvas),
            canvas_size: Some(canvas_size)
        }
    }
}

impl PluginGroup for EnginePlugins {
    fn build(self) -> PluginGroupBuilder {
        let (canvas_width, canvas_height) = self.canvas_size.unwrap_or((0.0, 0.0));

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
            .add(match self.canvas {
                Some(canvas) => WindowPlugin{
                    primary_window: Some(Window {
                        canvas: Some(canvas),
                        resolution: WindowResolution::new(canvas_width, canvas_height),
                        ..default()
                    }),
                    ..default()
                },
                None => WindowPlugin::default()
            })
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
    let build_web = true;
    let mut app = App::new();
    let width = 400.0;
    let height = 400.0;

    app.insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
        .insert_resource(Objects {
            count: 0,
            timer: Timer::from_seconds(0.1, TimerMode::Repeating)
        })
        .add_systems(Startup,load_startup_elements)
        .add_plugins(CollisionPlugin {
            sub_steps: 10,
            bounding_decay: Some(1.0),
            grid_center: Vec2::new(0.0, 0.0),
            grid_size: Vec2::new(width, height),
            grid_cell_size: Vec2::new(25.0, 25.0),
            debug_enabled: false,
            broad_mode: BroadPhaseMode::GridAccelerated,
            resolver_mode: ResolverMode::Momentum
        });

    if build_web {
        app.add_plugins(EnginePlugins::new(
            "#engine-canvas".into(),
            (width, height)
        ))
        .add_systems(Update,spawn_web_objects_elements);
    } else {
        app.add_plugins((
            GravityPlugin::default(),
            DebugLinesPlugin::default(),
            EnginePlugins::default()
        ))
        .add_systems(Update, input_update_step)
        .add_systems(Update,spawn_desktop_objects_elements);
    }

    app.run();
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
    mut commands: Commands
) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_web_objects_elements(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut objects: ResMut<Objects>
) {
    let colors = vec!["#ffca61", "#ffec85", "#c9f0d6"];
    objects.timer.tick(time.delta());

    if objects.timer.finished() && objects.count < 400 {
        objects.timer.reset();
        objects.count += 1;

        let radius = 10.0 * (0.5 + random::<f32>() / 2.0);
        let mass = 0.50 * radius * (random::<f32>() + 1.0);
        let circle = meshes.add(Mesh::from(Circle::new(radius))).into();

        let color = match Color::hex(colors[random::<usize>() % colors.len()]) {
            Ok(color) => color,
            Err(_) => Color::WHITE
        };

        let material = materials.add(ColorMaterial::from(color)).into();
        let pos = Transform::IDENTITY.with_translation(Vec3::new(
            100.0 * (2.0 * random::<f32>() - 1.0),
            100.0 * (2.0 * random::<f32>() - 1.0),
            0.0
        ));

        commands.spawn(Mesh2dColliderBundle {
            mesh: MaterialMesh2dBundle {
                mesh: circle,
                material: material,
                transform: pos,
                ..default()
            },
            physics: Physics::with_velocity(
                Vec2::ZERO,
                mass,
                pos,
                0.25 * Vec2::new(
                    2.0 * random::<f32>() - 1.0,
                    2.0 * random::<f32>() - 1.0
                )
            ),
            collider: CircleColliderBundle {
                aabb: AxisAlignedBoundingBox::new(
                    2.0 * Vec2::new(radius + 0.05, radius + 0.05)
                ),
                collider: CircleCollider {
                    radius: radius
                }
            }
        });
    }
}

fn spawn_desktop_objects_elements(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
    mut objects: ResMut<Objects>
) {
    let colors = vec!["#FF8578", "#FFB578", "#FFD86B", "8AFF8A", "#87E5FA", "#C58AFF", "#FC88D4"];
    objects.timer.tick(time.delta());

    if objects.timer.finished() && objects.count < 50 {
        objects.timer.reset();
        objects.count += 1;

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
            250.0 * (2.0 * random::<f32>() - 1.0),
            250.0 * (2.0 * random::<f32>() - 1.0),
            0.0
        ));

        commands.spawn(Mesh2dColliderBundle {
            mesh: MaterialMesh2dBundle {
                mesh: circle,
                material: material,
                transform: pos,
                ..default()
            },
            physics: Physics::with_velocity(
                Vec2::ZERO,
                mass,
                pos,
                0.15 * Vec2::new(
                    2.0 * random::<f32>() - 1.0,
                    2.0 * random::<f32>() - 1.0
                )
            ),
            collider: CircleColliderBundle {
                aabb: AxisAlignedBoundingBox::new(
                    2.0 * Vec2::new(radius + 0.05, radius + 0.05)
                ),
                collider: CircleCollider {
                    radius: radius
                }
            }
        });
    }
}