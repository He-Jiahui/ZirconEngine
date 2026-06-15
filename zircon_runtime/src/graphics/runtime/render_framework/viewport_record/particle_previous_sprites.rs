use crate::core::framework::render::RenderParticlePreviousSpriteSnapshot;

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn particle_previous_sprites(
        &self,
    ) -> &[RenderParticlePreviousSpriteSnapshot] {
        &self.particle_previous_sprites
    }

    pub(in crate::graphics::runtime::render_framework) fn replace_particle_previous_sprites(
        &mut self,
        sprites: Vec<RenderParticlePreviousSpriteSnapshot>,
    ) {
        self.particle_previous_sprites = sprites;
    }
}
