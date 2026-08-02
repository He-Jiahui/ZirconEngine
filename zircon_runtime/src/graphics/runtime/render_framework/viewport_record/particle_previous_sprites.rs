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
mod tests {
    use crate::core::framework::render::{
        CameraRenderDescriptor, RenderCameraTarget, RenderParticlePreviousSpriteSnapshot,
        RenderViewportDescriptor, RenderViewportRect, ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec2, Vec3};

    use super::super::camera_history_key::ViewportCameraHistoryKey;
    use super::ViewportRecord;

    #[test]
    fn viewport_record_keeps_particle_previous_sprites_per_camera_key() {
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
        let left_key = camera_key(1, UVec2::ZERO);
        let right_key = camera_key(1, UVec2::new(32, 0));

        record.replace_particle_previous_sprites(left_key.clone(), vec![previous_sprite(10)]);
        record.replace_particle_previous_sprites(right_key.clone(), vec![previous_sprite(20)]);

        assert_eq!(record.particle_previous_sprites(&left_key)[0].entity, 10);
        assert_eq!(record.particle_previous_sprites(&right_key)[0].entity, 20);
        assert!(record
            .particle_previous_sprites(&camera_key(2, UVec2::ZERO))
            .is_empty());
    }

    #[test]
    fn viewport_record_reuses_particle_previous_sprite_capacity_for_same_camera() {
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
        let key = camera_key(1, UVec2::ZERO);
        let mut sprites = Vec::with_capacity(8);
        sprites.push(previous_sprite(10));
        record.replace_particle_previous_sprites(key.clone(), sprites);

        let storage = record.particle_previous_sprites_for_update(key);
        let capacity = storage.capacity();
        storage.clear();
        storage.push(previous_sprite(20));

        assert!(capacity >= 8);
        assert_eq!(storage.capacity(), capacity);
    }

    fn previous_sprite(entity: u64) -> RenderParticlePreviousSpriteSnapshot {
        RenderParticlePreviousSpriteSnapshot {
            entity,
            stable_sprite_key: entity + 100,
            position: Vec3::new(entity as f32, 0.0, 0.0),
            size: 1.0,
            aspect_ratio: 1.0,
            billboard_offset: Vec2::ZERO,
            rotation: 0.0,
            billboard_basis: None,
        }
    }

    fn camera_key(entity: u64, position: UVec2) -> ViewportCameraHistoryKey {
        let mut descriptor = CameraRenderDescriptor::from_camera_payload(
            Some(entity),
            ViewportCameraSnapshot::default(),
        );
        descriptor.target = RenderCameraTarget::PrimarySurface;
        descriptor.viewport_rect = Some(RenderViewportRect::new(position, UVec2::new(32, 64)));
        ViewportCameraHistoryKey::from_camera(&descriptor)
    }
}
