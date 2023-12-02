use bevy::ecs::component::Component;
use bevy::math::Vec2;
use bevy::ecs::system::Query;
use bevy::prelude::Transform;

#[derive(Component)]
pub struct Collidable {
    pub bounds: Vec2,
}

pub fn collision_update_step(
    mut query: Query<(&mut Collidable, &mut Transform)>
) {
}
