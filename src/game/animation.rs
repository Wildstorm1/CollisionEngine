use std::time::Duration;
use bevy::ecs::component::Component;
use bevy::sprite::TextureAtlasSprite;
use bevy::time::{Time, Stopwatch};
use bevy::ecs::system::Query;
use bevy::prelude::Res;

// TODO: Should we have different components for different kinds of animations?
// Should different animations have different systems to drive them, ie couple
// the animation kind to the system? Or could we employ a trait and polymorphism
// with different logic? Does this couple things too much? Is it undoing the benefit
// of a composition model?
#[derive(Component)]
pub struct Animation {
    pub time: Stopwatch,
    pub period: Duration,
    pub frame_start: usize,
    pub frame_end: usize,
    pub frame: usize,
    pub restart: bool
}

pub fn animation_update_step(
    mut query: Query<(&mut TextureAtlasSprite, &mut Animation)>,
    time: Res<Time>
) {
    for (mut sprite, mut animation) in query.iter_mut() {
        animation.time.tick(time.delta());

        if animation.time.elapsed_secs_f64() > animation.period.as_secs_f64() {
            animation.time.reset();

            if animation.frame < animation.frame_end {
                animation.frame += 1;
            } else if animation.restart {
                animation.frame = animation.frame_start;
            }

            sprite.index = animation.frame;
        }        
    }
}
