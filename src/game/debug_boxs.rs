use super::bundles::CollisionEvent;
use super::collision::Collidable;
use std::collections::HashSet;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::EventReader;
use bevy::ecs::system::{Query, ResMut};
use bevy::math::Vec3;
use bevy::prelude::Transform;
use bevy::render::color::Color;
use bevy_prototype_debug_lines::DebugLines;

pub fn debug_boxs_update_step(
    mut query: Query<(Entity, &Collidable, &Transform)>,
    mut event_reader: EventReader<CollisionEvent>,
    mut lines: ResMut<DebugLines>
) {
    let mut highlight: HashSet<Entity> = HashSet::new();

    for event in event_reader.iter() {
        highlight.insert(event.entity);
        highlight.insert(event.other);
    }

    for (entity, collidable, transform) in query.iter_mut() {
        let debug_poly = vec![
            transform.translation + Vec3::new(-collidable.bounds.x / 2.0, collidable.bounds.y / 2.0, 0.0),
            transform.translation + Vec3::new(collidable.bounds.x / 2.0, collidable.bounds.y / 2.0, 0.0),
            transform.translation + Vec3::new(collidable.bounds.x / 2.0, -collidable.bounds.y / 2.0, 0.0),
            transform.translation + Vec3::new(-collidable.bounds.x / 2.0, -collidable.bounds.y / 2.0, 0.0)
        ];

        for i in 0..debug_poly.len() {
            let start = debug_poly[i];
            let end = debug_poly[(i + 1) % debug_poly.len()];
            let mut color = Color::WHITE;

            if highlight.contains(&entity) {
                color = Color::PINK;
            }

            lines.line_colored(start, end, 0.0, color);
        }
    }
}
