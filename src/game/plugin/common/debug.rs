use bevy::ecs::system::ResMut;
use bevy_prototype_debug_lines::DebugShapes;

pub trait DrawDebug {
    fn draw_debug(&self, shapes: &mut ResMut<DebugShapes>);
}