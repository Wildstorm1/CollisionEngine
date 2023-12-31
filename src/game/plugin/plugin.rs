use super::collision::list_grid::{ListGrid, GridSize};
use super::resolver::bounding_box::{ObjectBoundary, bound_in_box};
use super::resolver::fluid::fluid_resolver;
use super::resolver::speed::conserve_speed_resolver;
use super::resolver::momentum::conserve_momentum_resolver;
use super::physics::integrator::{Simulation, integration_phase};
use super::collision::cache::CollisionCache;
use super::collision::broad::{brute_force_broad_phase, accelerated_broad_phase};
use super::collision::narrow::narrow_phase;
use bevy::app::{Plugin, App, Update, PostUpdate};
use bevy::ecs::schedule::{IntoSystemConfigs, ScheduleLabel, Schedule, SystemSet, IntoSystemSetConfig};
use bevy::ecs::system::ResMut;
use bevy::ecs::world::World;
use bevy::log::error;
use bevy::math::Vec2;
use bevy::time::Time;
use bevy_prototype_debug_lines::DebugShapes;

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct CollisionSchedule;

pub enum BroadPhaseMode {
    BruteForce,
    GridAccelerated
}

pub enum ResolverMode {
    Fluid,
    Momentum,
    Speed
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum CollisionSet {
    Integration,
    BroadPhase,
    NarrowPhase,
    ResolverPhase
}

pub struct CollisionPlugin {
    pub sub_steps: usize,
    pub grid_center: Vec2,
    pub grid_size: Vec2,
    pub grid_cell_size: Vec2,
    pub debug_enabled: bool,
    pub broad_mode: BroadPhaseMode,
    pub resolver_mode: ResolverMode
}

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        let mut collision_schedule = Schedule::default();

        match self.broad_mode {
            BroadPhaseMode::BruteForce => {
                collision_schedule.add_systems(brute_force_broad_phase.in_set(CollisionSet::BroadPhase));
            },
            BroadPhaseMode::GridAccelerated => {
                collision_schedule.add_systems(accelerated_broad_phase.in_set(CollisionSet::BroadPhase));
            }
        }

        match self.resolver_mode {
            ResolverMode::Fluid => {
                collision_schedule.add_systems(fluid_resolver.in_set(CollisionSet::ResolverPhase));
            },
            ResolverMode::Momentum => {
                collision_schedule.add_systems(conserve_momentum_resolver.in_set(CollisionSet::ResolverPhase));
            },
            ResolverMode::Speed => {
                collision_schedule.add_systems(conserve_speed_resolver.in_set(CollisionSet::ResolverPhase));
            }
        }

        collision_schedule.add_systems((
            integration_phase.in_set(CollisionSet::Integration),
            narrow_phase.in_set(CollisionSet::NarrowPhase),
            bound_in_box.after(CollisionSet::ResolverPhase)
        ));

        let collision_schedule = collision_schedule;
        let cells = (self.grid_size / self.grid_cell_size).ceil() + 2.0;

        app.add_schedule(CollisionSchedule, collision_schedule)
        .configure_sets(CollisionSchedule, (
            CollisionSet::BroadPhase.after(CollisionSet::Integration),
            CollisionSet::NarrowPhase.after(CollisionSet::BroadPhase),
            CollisionSet::ResolverPhase.after(CollisionSet::NarrowPhase)
        ))
        .insert_resource(ObjectBoundary::new(
            self.grid_center,
            self.grid_size
        ))
        .insert_resource(Simulation::new(self.sub_steps))
        .insert_resource(CollisionCache::new(
            Some(Box::new(ListGrid::new(
                self.grid_center,
                GridSize {
                    width: cells.x as usize,
                    height: cells.y as usize
                },
                self.grid_cell_size,
            )))
        ))
        .add_systems(Update, run_schedule);

        if self.debug_enabled {
            app.add_systems(PostUpdate, draw_debug);
        }
    }
}

impl Default for CollisionPlugin {
    fn default() -> Self {
        CollisionPlugin {
            sub_steps: 8,
            grid_center: Vec2::new(0.0, 0.0),
            grid_size: Vec2::new(600.0, 600.0),
            grid_cell_size: Vec2::new(40.0, 40.0),
            debug_enabled: false,
            broad_mode: BroadPhaseMode::GridAccelerated,
            resolver_mode: ResolverMode::Fluid
        }
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

    let iters = match world.get_resource_mut::<Simulation>() {
        Some(mut simulation) => {
            simulation.set_time(time);
            simulation.iterations()
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