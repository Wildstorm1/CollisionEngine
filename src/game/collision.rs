use bevy::app::{App, Plugin, Update, PostUpdate};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::math::{Vec2, Vec3};
use bevy::ecs::system::{Query, ResMut, Resource, ParamSet, Res};
use bevy::prelude::Transform;
use bevy::render::color::Color;
use bevy::time::Time;
use bevy_prototype_debug_lines::{DebugShapes, DebugLines};
use std::hash::{Hash, Hasher};
use std::ops::BitXor;
use std::collections::{HashSet, HashMap};
use rand::random;

#[derive(Component)]
pub struct Physics {
    pub force: Vec2,
    pub mass: f32,
    prev_pos: Vec2
}

impl Physics {
    pub fn new(force: Vec2, mass: f32, position: Transform) -> Self {
        Physics {
            force: force,
            mass: mass,
            prev_pos: position.translation.truncate()
        }
    }
}

#[derive(Component)]
pub struct AxisAlignedBoundingBox {
    pub bounds: Vec2
}

#[derive(Component)]
pub struct CircleCollider {
    pub radius: f32
}

#[derive(Bundle)]
pub struct CircleColliderBundle {
    pub aabb: AxisAlignedBoundingBox,
    pub collider: CircleCollider
}

pub struct CollisionPlugin {
    pub enable_debug: bool,
    grid_size: f32,
    cell_size: f32
}

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        let debug_flag = self.enable_debug;

        let cells_x = (self.grid_size / self.cell_size) as usize;
        let cells_y = cells_x;
        let mut grid = Vec::with_capacity(cells_x * cells_y);

        for _ in 0..(cells_x * cells_y) {
            grid.push(Vec::with_capacity(1));
        }

        app.insert_resource(CollisionCache {
                potential_pairs: HashSet::with_capacity(200),
                live_pairs: HashMap::with_capacity(100),
                grid: grid,
                grid_root: Vec2::new(-self.grid_size / 2.0, -self.grid_size / 2.0),
                grid_size: Vec2::new(self.grid_size, self.grid_size),
                cell_size: Vec2::new(self.cell_size, self.cell_size),
                bounding_size: Vec2::new(self.grid_size - 2.0 * self.cell_size, self.grid_size - 2.0 * self.cell_size)
            })
            .add_systems(Update, run_step)
            .add_systems(PostUpdate, (
                debug_collision_update_step.run_if(move || debug_flag),
                debug_physics_update_step.run_if(move || debug_flag),
                debug_grid_update_step.run_if(move || debug_flag)
            ));
    }
}

impl Default for CollisionPlugin {
    fn default() -> Self {
        CollisionPlugin {
            enable_debug: false,
            grid_size: 2080.0,
            cell_size: 40.0
        }
    }
}

impl CollisionPlugin {
    pub fn with_debug() -> Self {
        CollisionPlugin {
            enable_debug: true,
            ..Default::default()
        }
    }
}

#[derive(Resource)]
struct CollisionCache {
    potential_pairs: HashSet<CollisionPair>,
    live_pairs: HashMap<CollisionPair, CollisionMetadata>,
    grid: Vec<Vec<Entity>>,
    grid_root: Vec2,
    grid_size: Vec2,
    cell_size: Vec2,
    bounding_size: Vec2
}

#[derive(Debug, Copy, Clone)]
struct CollisionPair {
    entity_a: Entity,
    entity_b: Entity
}

struct CollisionMetadata {
    is_first_frame: bool
}

impl Hash for CollisionPair {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.entity_a.to_bits().bitxor(self.entity_b.to_bits()).hash(state);
    }
}

impl PartialEq for CollisionPair {
    fn eq(&self, other: &Self) -> bool {
        (self.entity_a == other.entity_a && self.entity_b == other.entity_b) ||
        (self.entity_a == other.entity_b && self.entity_b == other.entity_a)
    }
}

impl Eq for CollisionPair {}

fn run_step(
    mut query_set: ParamSet<(
        Query<(&mut Physics, &mut Transform)>,
        Query<(Entity, &AxisAlignedBoundingBox, &Transform)>,
        Query<(&Transform, &CircleCollider)>,
        Query<(&mut Transform, &mut Physics, &CircleCollider)>
    )>,
    mut cache: ResMut<CollisionCache>,
    time: Res<Time>
) {
    let iters = 10;
    let dt = time.delta_seconds() / iters as f32;
    for i in 0..iters {
        integration_phase(&mut query_set.p0(), dt);
        resolve_collisions(&mut query_set, &mut cache);
    }
}

fn resolve_collisions(
    query_set: &mut ParamSet<(
        Query<(&mut Physics, &mut Transform)>,
        Query<(Entity, &AxisAlignedBoundingBox, &Transform)>,
        Query<(&Transform, &CircleCollider)>,
        Query<(&mut Transform, &mut Physics, &CircleCollider)>
    )>,
    cache: &mut ResMut<CollisionCache>
) {
    broad_phase(&query_set.p1(), cache);
    narrow_phase(&query_set.p2(), cache);
    resolve_phase(&mut query_set.p3(), cache);
}

fn integration_phase(
    query: &mut Query<(&mut Physics, &mut Transform)>,
    dt: f32
) {
    for (mut physics, mut transform) in query.iter_mut() {
        let cur_pos = transform.translation;
        let acceleration = physics.force / physics.mass;
        transform.translation = 2.0 * transform.translation - physics.prev_pos.extend(0.0) + acceleration.extend(0.0) * dt * dt;
        physics.prev_pos = cur_pos.truncate();
    }
}

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
fn broad_phase(
    query: &Query<(Entity, &AxisAlignedBoundingBox, &Transform)>,
    cache: &mut ResMut<CollisionCache>
) {
    let CollisionCache {
        potential_pairs,
        grid,
        grid_size,
        cell_size,
        ..
    } = cache.as_mut();

    let cells_x = (grid_size.x / cell_size.x) as usize;
    let cells_y = (grid_size.y / cell_size.y) as usize;

    grid.iter_mut().for_each(|cell| { cell.drain(0..); () });

    for (entity, _, transform) in query.iter() {
        let coord = ((transform.translation.truncate() + (*grid_size / 2.0)) / *cell_size).floor();
        let cell_x = coord.x as usize;
        let cell_y = coord.y as usize;
        let i = cell_x + cell_y * cells_x;
        grid[i].push(entity);
    }

    let mut seen_pairs = HashSet::new();

    for (entity_a, aabb_a, transform_a) in query.iter() {
        let coord = ((transform_a.translation.truncate() + (*grid_size / 2.0)) / *cell_size).floor();
        let cell_x = coord.x as usize;
        let cell_y = coord.y as usize;

        //for x in cell_x.saturating_sub(1)..(cell_x + 1).min(cells_x) {
        //    for y in cell_y.saturating_sub(1)..(cell_y + 1).min(cells_y) {
        for xi in -1..=1 {
            for yi in -1..=1 {
                if cell_x as i32 + xi < 0 || cell_x as i32 + xi >= cells_x as i32 || cell_y as i32 + yi < 0 || cell_y as i32 + yi >= cells_y as i32 {
                    continue;
                }

                let x = (cell_x as i32 + xi) as usize;
                let y = (cell_y as i32 + yi) as usize;
                let i = x + y * cells_x;

                for cell_entity in grid[i].iter() {
                    let (entity_b, aabb_b, transform_b) = match query.get(*cell_entity) {
                        Ok(entity) => entity,
                        Err(_) => continue
                    };

                    let pair = CollisionPair {
                        entity_a: entity_a,
                        entity_b: entity_b
                    };

                    if entity_a == entity_b || seen_pairs.contains(&pair) {
                        continue;
                    }

                    // TODO: SIMD, Can remove the divide by 2.0
                    let half_bound_a = aabb_a.bounds / 2.0;
                    let half_bound_b = aabb_b.bounds / 2.0;
                    let translation_a = transform_a.translation.truncate();
                    let translation_b = transform_b.translation.truncate();
                    let a_min = translation_a - half_bound_a;
                    let a_max = translation_a + half_bound_a;
                    let b_min = translation_b - half_bound_b;
                    let b_max = translation_b + half_bound_b;

                    if a_min.x > b_max.x || a_max.x < b_min.x || a_min.y > b_max.y || a_max.y < b_min.y {
                        continue;
                    }

                    potential_pairs.insert(pair);
                    seen_pairs.insert(pair);
                }
            }
        }
    }
}

fn narrow_phase(
    query: &Query<(&Transform, &CircleCollider)>,
    cache: &mut ResMut<CollisionCache>
) {
    let CollisionCache {
        potential_pairs,
        live_pairs,
        ..
    } = cache.as_mut();
    live_pairs.retain(|pair, _| potential_pairs.contains(pair));

    for pair in potential_pairs.drain() {
        let (transform_a, collidable_a) = match query.get(pair.entity_a) {
            Ok(entity) => entity,
            Err(_) => continue
        };

        let (transform_b, collidable_b) = match query.get(pair.entity_b) {
            Ok(entity) => entity,
            Err(_) => continue
        };

        let are_colliding = transform_a.translation.truncate().distance(transform_b.translation.truncate()) < collidable_a.radius + collidable_b.radius;
        let in_cache = live_pairs.contains_key(&pair);

        if are_colliding && !in_cache {
            live_pairs.insert(pair, CollisionMetadata { is_first_frame: true });
        } else if !are_colliding && in_cache {
            live_pairs.remove(&pair);
        } else {
            match live_pairs.get_mut(&pair) {
                Some(metadata) => {
                    metadata.is_first_frame = false;
                },
                None => {}
            }
        }
    }
}

fn resolve_phase(
    query: &mut Query<(&mut Transform, &mut Physics, &CircleCollider)>,
    cache: &mut ResMut<CollisionCache>
) {
    let CollisionCache {
        live_pairs,
        bounding_size,
        ..
    } = cache.as_mut();

    for (pair, metadata) in live_pairs.iter_mut() {
        let [
            (mut transform_a, mut physics_a, collidable_a),
            (mut transform_b, mut physics_b, collidable_b)
        ] = match query.get_many_mut([pair.entity_a, pair.entity_b]) {
            Ok(entities) => entities,
            Err(_) => continue
        };

        let mut separation_vector = transform_b.translation - transform_a.translation;
        let are_colliding = transform_a.translation.distance(transform_b.translation) < collidable_a.radius + collidable_b.radius;

        if !are_colliding {
            continue;
        }

        let overlap = (collidable_a.radius + collidable_b.radius - separation_vector.length()) / 2.0;

        if overlap <= f32::EPSILON {
            separation_vector = Vec3::new(2.0 * random::<f32>() - 1.0, 2.0 * random::<f32>() - 1.0, 0.0);
        }

        let speed_a = 0.90 * (transform_a.translation.truncate() - physics_a.prev_pos).length();
        let speed_b = 0.90 * (transform_b.translation.truncate() - physics_b.prev_pos).length();

        let resolve_direction = separation_vector.normalize();
        let resolve_vector = overlap * resolve_direction;
        transform_b.translation += resolve_vector;
        physics_b.prev_pos = (transform_b.translation - speed_b * resolve_direction).truncate();
        transform_a.translation -= resolve_vector;
        physics_a.prev_pos = (transform_a.translation + speed_a * resolve_direction).truncate();

        if metadata.is_first_frame {
            metadata.is_first_frame = false;
        }
    }

    let lower = (-1.0 * *bounding_size / 2.0).extend(0.0);
    let upper = (*bounding_size / 2.0).extend(0.0);

    for entity in query.iter_mut() {
        let (mut transform, _, _) = entity;
        transform.translation = transform.translation.clamp(lower, upper);
    }
}

fn debug_collision_update_step(
    query: Query<(&AxisAlignedBoundingBox, &Transform)>,
    mut shapes: ResMut<DebugShapes>
) {
    for (aabb, transform) in query.iter() {
        shapes.rect()
            .position(transform.translation)
            .size(aabb.bounds)
            .color(Color::WHITE);
    }
}

fn debug_grid_update_step(
    cache: ResMut<CollisionCache>,
    mut shapes: ResMut<DebugShapes>
) {
    let cells_x = (cache.grid_size.x / cache.cell_size.x) as usize;
    let cells_y = (cache.grid_size.y / cache.cell_size.y) as usize;
    let render_size = cache.cell_size - Vec2::new(0.5, 0.5);

    for cell_x in 0..cells_x {
        for cell_y in 0..cells_y {
            let i = cell_x + cell_y * cells_x;
            let mut color = Color::WHITE;

            if cache.grid[i].len() > 0 {
                color = Color::RED;
            }

            shapes.rect()
                .position((cache.grid_root + (cache.cell_size / 2.0) + Vec2::new(cell_x as f32, cell_y as f32) * cache.cell_size).extend(0.0))
                .size(render_size)
                .color(color);
        }
    }
}

fn debug_physics_update_step(
    query: Query<(&Physics, &Transform)>,
    mut lines: ResMut<DebugLines>
) {
    for (physics, transform) in query.iter() {
        lines.line_colored(
            transform.translation,
            transform.translation + physics.force.extend(0.0),
            0.0,
            Color::RED
        );
    }
}
