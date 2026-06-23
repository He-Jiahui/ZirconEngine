use crate::core::framework::render::RenderParticlePreviousSpriteSnapshot;
use crate::graphics::ViewportRenderFrame;

use super::super::super::viewport_record::{ViewportCameraHistoryKey, ViewportRecord};

pub(super) fn update_particle_previous_state_after_success(
    record: &mut ViewportRecord,
    frame: &ViewportRenderFrame,
    camera_history_key: &ViewportCameraHistoryKey,
) {
    let camera = frame.effective_camera().transform;
    let right = camera.right();
    let up = camera.up();
    let ambiguous_anonymous_entities = frame
        .extract
        .particles
        .anonymous_stream_ambiguity_entities();
    record.replace_particle_previous_sprites(
        camera_history_key.clone(),
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
            })
            .collect(),
    );
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        RenderFrameExtract, RenderLayerSet, RenderParticleBillboardBasisSnapshot,
        RenderParticleSpriteSnapshot, RenderViewportDescriptor, RenderWorldSnapshotHandle,
    };
    use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
    use crate::graphics::runtime::render_framework::viewport_record::{
        ViewportCameraHistoryKey, ViewportRecord,
    };
    use crate::graphics::ViewportRenderFrame;
    use crate::scene::world::World;

    use super::update_particle_previous_state_after_success;

    #[test]
    fn successful_submit_records_particle_previous_state_for_next_frame() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(9),
            World::new().to_render_snapshot(),
        );
        extract.view.camera.transform = Transform::from_translation(Vec3::new(0.0, 0.0, 4.0));
        extract.particles.sprites = vec![RenderParticleSpriteSnapshot {
            entity: 77,
            stable_sprite_key: 31,
            position: Vec3::new(1.0, 2.0, 3.0),
            size: 0.75,
            aspect_ratio: 1.5,
            billboard_offset: Vec2::new(0.1, -0.2),
            rotation: 0.25,
            sort_order: 3,
            color: Vec4::ONE,
            intensity: 1.0,
            depth_test: true,
            render_layer_mask: RenderLayerSet::from_legacy_mask(u32::MAX),
            material: None,
            texture: None,
        }];
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));

        let key = ViewportCameraHistoryKey::from_camera(frame.camera());

        update_particle_previous_state_after_success(&mut record, &frame, &key);

        assert_eq!(record.particle_previous_sprites(&key).len(), 1);
        let previous = record.particle_previous_sprites(&key)[0];
        assert_eq!(previous.entity, 77);
        assert_eq!(previous.stable_sprite_key, 31);
        assert_eq!(previous.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(previous.size, 0.75);
        assert_eq!(previous.aspect_ratio, 1.5);
        assert_eq!(previous.billboard_offset, Vec2::new(0.1, -0.2));
        assert_eq!(previous.rotation, 0.25);
        assert_eq!(
            previous.billboard_basis,
            Some(RenderParticleBillboardBasisSnapshot::new(
                frame.effective_camera().transform.right(),
                frame.effective_camera().transform.up(),
            ))
        );
    }

    #[test]
    fn successful_submit_drops_ambiguous_anonymous_particle_previous_state() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(9),
            World::new().to_render_snapshot(),
        );
        extract.view.camera.transform = Transform::from_translation(Vec3::new(0.0, 0.0, 4.0));
        extract.particles.sprites = vec![
            RenderParticleSpriteSnapshot {
                entity: 77,
                stable_sprite_key: 0,
                position: Vec3::new(1.0, 2.0, 3.0),
                size: 0.75,
                aspect_ratio: 1.5,
                billboard_offset: Vec2::new(0.1, -0.2),
                rotation: 0.25,
                sort_order: 3,
                color: Vec4::ONE,
                intensity: 1.0,
                depth_test: true,
                render_layer_mask: RenderLayerSet::from_legacy_mask(u32::MAX),
                material: None,
                texture: None,
            },
            RenderParticleSpriteSnapshot {
                entity: 77,
                stable_sprite_key: 0,
                position: Vec3::new(2.0, 2.0, 3.0),
                size: 0.75,
                aspect_ratio: 1.5,
                billboard_offset: Vec2::new(0.1, -0.2),
                rotation: 0.25,
                sort_order: 4,
                color: Vec4::ONE,
                intensity: 1.0,
                depth_test: true,
                render_layer_mask: RenderLayerSet::from_legacy_mask(u32::MAX),
                material: None,
                texture: None,
            },
            RenderParticleSpriteSnapshot {
                entity: 78,
                stable_sprite_key: 0,
                position: Vec3::new(3.0, 2.0, 3.0),
                size: 0.75,
                aspect_ratio: 1.5,
                billboard_offset: Vec2::new(0.1, -0.2),
                rotation: 0.25,
                sort_order: 5,
                color: Vec4::ONE,
                intensity: 1.0,
                depth_test: true,
                render_layer_mask: RenderLayerSet::from_legacy_mask(u32::MAX),
                material: None,
                texture: None,
            },
        ];
        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));

        let key = ViewportCameraHistoryKey::from_camera(frame.camera());

        update_particle_previous_state_after_success(&mut record, &frame, &key);

        assert_eq!(record.particle_previous_sprites(&key).len(), 1);
        assert_eq!(record.particle_previous_sprites(&key)[0].entity, 78);
    }
}
