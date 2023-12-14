use bevy::ecs::bundle::Bundle;
use bevy::sprite::{MaterialMesh2dBundle, Material2d};

#[derive(Bundle)]
pub struct Mesh2dColliderBundle<M> where M: Material2d {
    pub mesh: MaterialMesh2dBundle<M>
}
