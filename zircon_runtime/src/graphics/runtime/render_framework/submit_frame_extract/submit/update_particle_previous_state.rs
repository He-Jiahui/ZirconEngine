use crate::core::framework::render::RenderParticlePreviousSpriteSnapshot;
use crate::graphics::ViewportRenderFrame;

use super::super::super::viewport_record::{ViewportCameraHistoryKey, ViewportRecord};

pub(super) fn update_particle_previous_state_after_success(
    record: &mut ViewportRecord,
    frame: &mut ViewportRenderFrame,
    camera_history_key: &ViewportCameraHistoryKey,
) {
    let camera = frame.effective_camera().transform;
    let right = camera.right();
    let up = camera.up();
    let ambiguous_anonymous_entities = frame
        .extract
        .particles
        .anonymous_stream_ambiguity_entities();
    let recycled_previous_sprites = frame.take_particle_previous_sprites_override();
    let rebuild_previous_sprites = |previous_sprites: &mut Vec<_>| {
        previous_sprites.clear();
        previous_sprites.reserve(frame.extract.particles.sprites.len());
        previous_sprites.extend(
            frame
                .extract
                .particles
                .sprites
                .iter()
                .filter(|sprite| {
                    sprite.stable_sprite_key != 0
                        || !ambiguous_anonymous_entities.contains(&sprite.entity)
                })
                .map(|sprite| {
                    RenderParticlePreviousSpriteSnapshot::from_current_with_billboard_basis(
                        sprite, right, up,
                    )
                }),
        );
    };
    if let Some(mut recycled_previous_sprites) = recycled_previous_sprites {
        rebuild_previous_sprites(&mut recycled_previous_sprites);
        *record.particle_previous_sprites_for_update(camera_history_key.clone()) =
            recycled_previous_sprites;
    } else {
        rebuild_previous_sprites(
            record.particle_previous_sprites_for_update(camera_history_key.clone()),
        );
    }
}

#[cfg(test)]
#[path = "tests/update_particle_previous_state.rs"]
mod tests;
