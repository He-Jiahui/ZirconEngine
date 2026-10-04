use crate::core::framework::render::RenderParticlePreviousSpriteSnapshot;

use super::{viewport_record::ViewportRecord, ViewportCameraHistoryKey};

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn particle_previous_sprites(
        &self,
        key: &ViewportCameraHistoryKey,
    ) -> &[RenderParticlePreviousSpriteSnapshot] {
        self.particle_previous_sprites
            .get(key)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub(in crate::graphics::runtime::render_framework) fn replace_particle_previous_sprites(
        &mut self,
        key: ViewportCameraHistoryKey,
        sprites: Vec<RenderParticlePreviousSpriteSnapshot>,
    ) {
        self.particle_previous_sprites.insert(key, sprites);
    }

    pub(in crate::graphics::runtime::render_framework) fn particle_previous_sprites_for_update(
        &mut self,
        key: ViewportCameraHistoryKey,
    ) -> &mut Vec<RenderParticlePreviousSpriteSnapshot> {
        self.particle_previous_sprites.entry(key).or_default()
    }
}

#[cfg(test)]
#[path = "tests/particle_previous_sprites.rs"]
mod tests;
