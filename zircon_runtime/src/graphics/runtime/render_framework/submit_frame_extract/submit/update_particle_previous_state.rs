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
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::framework::render::{
        RenderFrameExtract, RenderLayerSet, RenderParticleBillboardBasisSnapshot,
        RenderParticlePreviousSpriteSnapshot, RenderParticleSpriteSnapshot,
        RenderViewportDescriptor, RenderWorldSnapshotHandle,
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
            render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            material: None,
            texture: None,
        }];
        let mut frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));

        let key = ViewportCameraHistoryKey::from_camera(frame.camera());

        update_particle_previous_state_after_success(&mut record, &mut frame, &key);

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
    fn successful_submit_recycles_renderer_owned_particle_history_capacity() {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(9),
            World::new().to_render_snapshot(),
        );
        extract.particles.sprites = vec![RenderParticleSpriteSnapshot {
            entity: 77,
            stable_sprite_key: 31,
            ..RenderParticleSpriteSnapshot::default()
        }];
        let mut previous_sprites = Vec::with_capacity(8);
        previous_sprites.push(RenderParticlePreviousSpriteSnapshot::default());
        let previous_capacity = previous_sprites.capacity();
        let previous_pointer = previous_sprites.as_ptr();
        let mut frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64))
            .with_particle_previous_sprites_override(Some(previous_sprites));
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));
        let key = ViewportCameraHistoryKey::from_camera(frame.camera());

        update_particle_previous_state_after_success(&mut record, &mut frame, &key);

        let recorded = record.particle_previous_sprites(&key);
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded.capacity(), previous_capacity);
        assert_eq!(recorded.as_ptr(), previous_pointer);
        assert!(frame.previous_particle_sprites().is_empty());
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
                render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
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
                render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
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
                render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                material: None,
                texture: None,
            },
        ];
        let mut frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));
        let mut record = ViewportRecord::new(RenderViewportDescriptor::new(UVec2::new(64, 64)));

        let key = ViewportCameraHistoryKey::from_camera(frame.camera());

        update_particle_previous_state_after_success(&mut record, &mut frame, &key);

        assert_eq!(record.particle_previous_sprites(&key).len(), 1);
        assert_eq!(record.particle_previous_sprites(&key)[0].entity, 78);
    }

    #[test]
    fn optimization_batch_dp_previous_particle_state_reserves_source_count() {
        let source = include_str!("update_particle_previous_state.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("particle previous state production source");
        assert!(
            production.contains("previous_sprites.reserve(frame.extract.particles.sprites.len())")
        );
    }

    #[test]
    #[ignore = "release-only alternating p95 performance gate"]
    fn optimization_batch_dp_previous_particle_state_capacity_p95() {
        const SAMPLE_PAIRS: usize = 17;
        const BUILDS_PER_SAMPLE: usize = 8_192;
        const SPRITES_PER_BUILD: usize = 512;

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_previous_sprite_capacity(
                    BUILDS_PER_SAMPLE,
                    SPRITES_PER_BUILD,
                    false,
                ));
                optimized_samples.push(measure_previous_sprite_capacity(
                    BUILDS_PER_SAMPLE,
                    SPRITES_PER_BUILD,
                    true,
                ));
            } else {
                optimized_samples.push(measure_previous_sprite_capacity(
                    BUILDS_PER_SAMPLE,
                    SPRITES_PER_BUILD,
                    true,
                ));
                legacy_samples.push(measure_previous_sprite_capacity(
                    BUILDS_PER_SAMPLE,
                    SPRITES_PER_BUILD,
                    false,
                ));
            }
        }

        let legacy_p95 = p95(&mut legacy_samples);
        let optimized_p95 = p95(&mut optimized_samples);
        println!(
            "RUNTIME425_PREVIOUS_PARTICLE_STATE_CAPACITY_BENCH_V1 builds_per_sample={BUILDS_PER_SAMPLE} sprites_per_build={SPRITES_PER_BUILD} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} ratio={:.4}",
            optimized_p95 as f64 / legacy_p95.max(1) as f64
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "previous particle state capacity p95 {optimized_p95}ns exceeded 70% of legacy {legacy_p95}ns"
        );

        fn measure_previous_sprite_capacity(
            build_count: usize,
            sprite_count: usize,
            reserve: bool,
        ) -> u128 {
            let started_at = Instant::now();
            let mut checksum = 0_usize;
            for build_index in 0..build_count {
                let mut previous_sprites = Vec::new();
                if reserve {
                    previous_sprites.reserve(sprite_count);
                }
                previous_sprites.extend((0..sprite_count).map(|sprite| sprite ^ build_index));
                checksum =
                    checksum.wrapping_add(previous_sprites.len() ^ previous_sprites.capacity());
                black_box(&previous_sprites);
            }
            black_box(checksum);
            started_at.elapsed().as_nanos()
        }

        fn p95(samples: &mut [u128]) -> u128 {
            samples.sort_unstable();
            samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
        }
    }
}
