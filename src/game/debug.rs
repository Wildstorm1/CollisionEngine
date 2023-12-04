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

pub fn debug_update_step(
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
        //let debug_poly = construct_box(collidable, transform);
        let debug_circle = construct_circle(collidable, transform, 20);
        let mut color = Color::WHITE;

        if highlight.contains(&entity) {
            color = Color::PINK;
        }

        //draw_shape(&mut lines, debug_poly, Color::WHITE);
        draw_shape(&mut lines, debug_circle, color);
    }
}

fn construct_circle(
    collidable: &Collidable,
    transform: &Transform,
    points: u32
) -> Vec<Vec3> {
    let mut circle = Vec::new();

    for i in 0..points {
        let angle = (i as f32) * 2.0 * std::f32::consts::PI / (points as f32);
        let point = transform.rotation.mul_vec3(collidable.collider.radius * Vec3::new(angle.cos(), angle.sin(), 0.0));
        circle.push(transform.translation + point);
    }

    circle
}

fn construct_box(
    collidable: &Collidable,
    transform: &Transform
) -> Vec<Vec3> {
    vec![
        transform.translation + Vec3::new(-collidable.bounds.x / 2.0, collidable.bounds.y / 2.0, 0.0),
        transform.translation + Vec3::new(collidable.bounds.x / 2.0, collidable.bounds.y / 2.0, 0.0),
        transform.translation + Vec3::new(collidable.bounds.x / 2.0, -collidable.bounds.y / 2.0, 0.0),
        transform.translation + Vec3::new(-collidable.bounds.x / 2.0, -collidable.bounds.y / 2.0, 0.0)
    ]
}

fn draw_shape(
    lines: &mut ResMut<DebugLines>,
    polygon: Vec<Vec3>,
    color: Color
) {
    for i in 0..polygon.len() {
        let start = polygon[i];
        let end = polygon[(i + 1) % polygon.len()];

        lines.line_colored(start, end, 0.0, color);
    }
}
