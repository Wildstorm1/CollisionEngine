use super::bundles::Ship;
use super::physics::Physics;
use bevy::ecs::system::Query;
use bevy::ecs::query::With;
use bevy::prelude::Res;
use bevy::hierarchy::Children;
use bevy::render::view::Visibility;
use bevy::sprite::TextureAtlasSprite;
use bevy::time::Time;

pub fn ship_state_update_step(
    mut query: Query<(&mut Ship, &Physics, &Children)>,
    mut sprites: Query<&mut Visibility, With<TextureAtlasSprite>>,
    time: Res<Time>
) {
    for (mut ship, physics, children) in query.iter_mut() {
        ship.reload.tick(time.delta());

        // A ship will have a small number of children, so we loop over them and look them up in
        // the more general query of all exhaust sprites.
        // We could make the sprites query more specific by tagging exhaust with a tag
        // Also, if no children match the query, this section does nothing
        for &child in children.iter() {
            match sprites.get_mut(child) {
                Ok(mut sprite) => {
                    if physics.acceleration.length() < 3.0 * f32::EPSILON {
                        *sprite = Visibility::Hidden;
                    } else {
                        *sprite = Visibility::Visible;
                    }
                },
                Err(_) => {}
            }
        }
    }
}
