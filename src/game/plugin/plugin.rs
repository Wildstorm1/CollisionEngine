use super::collision::list_grid::{ListGrid, GridSize};
use super::resolver::resolver::{ObjectBoundary, basic_transform_resolve_phase};
use super::physics::integrator::{Simulation, integration_phase};
use super::collision::cache::CollisionCache;
use super::collision::broad::{brute_force_broad_phase, accelerated_broad_phase};
use super::collision::narrow::narrow_phase;
use bevy::app::{Plugin, App, Update};
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Res, ResMut};
use bevy::math::Vec2;
use bevy::time::Time;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ObjectBoundary::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(600.0, 600.0)
        ))
        .insert_resource(Simulation::new())
        .insert_resource(CollisionCache::with_capacity(
            600,
            Some(Box::new(ListGrid::new(
                Vec2::new(0.0, 0.0),
                GridSize { width: 640, height: 640 },
                Vec2::new(40.0, 40.0),
            )))
        ))
        .add_systems(Update, (
            update_simulation_time,
            integration_phase.after(update_simulation_time),
            //.after(integration_phase),
            //narrow_phase.after(accelerated_broad_phase),
            brute_force_broad_phase.after(integration_phase),
            narrow_phase.after(brute_force_broad_phase),
            basic_transform_resolve_phase.after(narrow_phase)
        ));
    }
}

impl Default for CollisionPlugin {
    fn default() -> Self {
        CollisionPlugin {}
    }
}

fn update_simulation_time(
    time: Res<Time>,
    mut simulation: ResMut<Simulation>
) {
    simulation.set_time(time.delta_seconds());
}