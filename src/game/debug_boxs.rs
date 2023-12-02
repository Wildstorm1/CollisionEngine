use super::collision::Collidable;
use bevy::ecs::system::{Query, ResMut};
use bevy::math::Vec3;
use bevy::prelude::Transform;
use bevy_prototype_debug_lines::DebugLines;

pub fn debug_boxs_update_step(
    mut query: Query<(&Collidable, &Transform)>,
    mut lines: ResMut<DebugLines>
) {
    for (collidable, transform) in query.iter_mut() {
        let debug_poly = vec![
            transform.translation + Vec3::new(-collidable.bounds.x / 2.0, collidable.bounds.y / 2.0, 0.0),
            transform.translation + Vec3::new(collidable.bounds.x / 2.0, collidable.bounds.y / 2.0, 0.0),
            transform.translation + Vec3::new(collidable.bounds.x / 2.0, -collidable.bounds.y / 2.0, 0.0),
            transform.translation + Vec3::new(-collidable.bounds.x / 2.0, -collidable.bounds.y / 2.0, 0.0)
        ];

        for i in 0..debug_poly.len() {
            let start = debug_poly[i];
            let end = debug_poly[(i + 1) % debug_poly.len()];
            lines.line(start, end, 0.0);
        }
    }
}
