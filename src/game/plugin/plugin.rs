use super::collision::list_grid::{ListGrid, GridSize};
use super::resolver::resolver::{ObjectBoundary, basic_transform_resolve_phase};
use super::physics::integrator::{Simulation, integration_phase};
use super::collision::cache::CollisionCache;
use super::collision::broad::{brute_force_broad_phase, accelerated_broad_phase};
use super::collision::narrow::narrow_phase;
use bevy::app::{Plugin, App, Update, PostUpdate};
use bevy::ecs::schedule::{IntoSystemConfigs, ScheduleLabel, Schedule};
use bevy::ecs::system::{Res, ResMut};
use bevy::ecs::world::World;
use bevy::log::error;
use bevy::math::Vec2;
use bevy::time::Time;
use bevy_prototype_debug_lines::DebugShapes;

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct CollisionSchedule;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        let mut collision_schedule = Schedule::default();

        collision_schedule.add_systems((
            integration_phase,
            accelerated_broad_phase.after(integration_phase),
            narrow_phase.after(accelerated_broad_phase),
            //brute_force_broad_phase.after(integration_phase),
            //narrow_phase.after(brute_force_broad_phase),
            basic_transform_resolve_phase.after(narrow_phase)
        ));

        let collision_schedule = collision_schedule;

        app.add_schedule(CollisionSchedule, collision_schedule)
        .insert_resource(ObjectBoundary::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(600.0, 600.0)
        ))
        .insert_resource(Simulation::new())
        .insert_resource(CollisionCache::with_capacity(
            600,
            Some(Box::new(ListGrid::new(
                Vec2::new(0.0, 0.0),
                GridSize { width: 16, height: 16 },
                Vec2::new(40.0, 40.0),
            )))
        ))
        .add_systems(Update, run_schedule);
        //.add_systems(PostUpdate, draw_debug);
    }
}

impl Default for CollisionPlugin {
    fn default() -> Self {
        CollisionPlugin {}
    }
}

fn run_schedule(
    world: &mut World
) {
    let time = match world.get_resource::<Time>() {
        Some(time) => time.delta_seconds(),
        None => {
            error!("Time resource not found");
            return;
        }
    };

    let iters = 15;

    match world.get_resource_mut::<Simulation>() {
        Some(mut simulation) => {
            simulation.set_time(time / iters as f32);
        },
        None => {
            error!("Simulation resource not found");
            return;
        }
    };

    for _ in 0..iters {
        world.run_schedule(CollisionSchedule);
    }
}

fn draw_debug(
    mut shapes: ResMut<DebugShapes>,
    cache: ResMut<CollisionCache>
) {
    match cache.accelerator {
        Some(ref accelerator) => accelerator.draw_debug(&mut shapes),
        None => {}
    }
}