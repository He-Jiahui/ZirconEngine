use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    CorePipelineKind, GeometryExtract, ProjectionMode, RenderFrameExtract, RenderFramework,
    RenderLayerSet, RenderMaterialAlphaMode, RenderParticleSpriteSnapshot, RenderPhase,
    RenderPhaseMeshSource, RenderPipelineHandle, RenderQualityProfile, RenderQueueValue,
    RenderSpriteAnchor, RenderSpriteImageMode, RenderSpriteSnapshot, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, RendererCommon, SpriteExtract,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ResourceHandle, ResourceId, TextureMarker};
use crate::graphics::WgpuRenderFramework;

#[test]
fn render_product_sprite_contract_is_distinct_from_particle_sprites() {
    let sprite = RenderSpriteSnapshot {
        entity: 10,
        transform: Transform::from_translation(Vec3::new(1.0, 2.0, 3.0)),
        image: texture_handle("res://textures/hero.png"),
        material: Some(material_handle("res://materials/sprite.zmaterial")),
        atlas_region: Some(crate::core::framework::render::RenderSpriteAtlasRegion {
            min: Vec2::new(0.25, 0.5),
            max: Vec2::new(0.5, 0.75),
        }),
        rect: Some(crate::core::framework::render::RenderSpriteRect {
            min: Vec2::new(4.0, 8.0),
            max: Vec2::new(20.0, 40.0),
        }),
        flip_x: true,
        flip_y: false,
        anchor: RenderSpriteAnchor::TOP_LEFT,
        custom_size: Some(Vec2::new(2.0, 4.0)),
        image_mode: RenderSpriteImageMode::Stretch,
        color: Vec4::new(0.5, 0.75, 1.0, 0.6),
        z_order: 7,
        common: RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(0b10),
            ..RendererCommon::default()
        },
        material_alpha_mode: RenderMaterialAlphaMode::Blend,
    };
    let particle = RenderParticleSpriteSnapshot {
        entity: sprite.entity,
        stable_sprite_key: 1,
        position: sprite.transform.translation,
        size: 4.0,
        aspect_ratio: 1.0,
        billboard_offset: Vec2::ZERO,
        rotation: 0.0,
        sort_order: 0,
        color: sprite.color,
        intensity: 1.0,
        depth_test: true,
        render_layer_mask: sprite.common.layer_mask.clone(),
        material: sprite.material,
        texture: Some(sprite.image),
    };
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(77),
        super::render_product_submit::snapshot_with_projection_for_sprite_tests(
            ProjectionMode::Orthographic,
        ),
    );
    extract.sprites.sprites.push(sprite.clone());
    extract.particles.sprites.push(particle);

    assert_eq!(extract.sprites.sprites, vec![sprite]);
    assert_eq!(extract.particles.sprites.len(), 1);
    assert_ne!(extract.sprites.sprites.len(), 0);
}

#[test]
fn render_product_sprite_snapshot_hard_cuts_legacy_layer_field_into_renderer_common() {
    let source = include_str!("../../core/framework/render/sprite/sprite.rs");

    assert!(source.contains("pub common: RendererCommon"));
    assert!(!source.contains("pub render_layer_mask: RenderLayerSet"));
}

#[test]
fn render_product_sprite_phase_queue_uses_core2d_phase_order_and_transparent_depth_sort() {
    let queue = crate::core::framework::render::build_sprite_phase_queue(
        CorePipelineKind::Core2d,
        [
            crate::core::framework::render::SpritePhaseInput::new(
                30,
                0,
                RenderQueueValue::TRANSPARENT,
                2,
                2.0,
            ),
            crate::core::framework::render::SpritePhaseInput::new(
                10,
                1,
                RenderQueueValue::GEOMETRY,
                0,
                1.0,
            ),
            crate::core::framework::render::SpritePhaseInput::new(
                20,
                2,
                RenderQueueValue::ALPHA_TEST,
                1,
                3.0,
            ),
            crate::core::framework::render::SpritePhaseInput::new(
                40,
                3,
                RenderQueueValue::TRANSPARENT,
                1,
                4.0,
            ),
        ],
    );

    assert_eq!(
        queue
            .items
            .iter()
            .map(|item| item.phase)
            .collect::<Vec<_>>(),
        vec![
            RenderPhase::Opaque2d,
            RenderPhase::AlphaMask2d,
            RenderPhase::Transparent2d,
            RenderPhase::Transparent2d,
        ]
    );
    assert_eq!(
        queue
            .items
            .iter()
            .map(|item| item.mesh_source)
            .collect::<Vec<_>>(),
        vec![
            RenderPhaseMeshSource::SpriteIndex(1),
            RenderPhaseMeshSource::SpriteIndex(2),
            RenderPhaseMeshSource::SpriteIndex(3),
            RenderPhaseMeshSource::SpriteIndex(0),
        ]
    );
}

#[test]
fn render_product_sprite_phase_queue_honors_queue_and_order_in_layer() {
    let queue = crate::core::framework::render::build_sprite_phase_queue(
        CorePipelineKind::Core2d,
        [
            crate::core::framework::render::SpritePhaseInput::new(
                30,
                0,
                RenderQueueValue::GEOMETRY.with_material_offset_i32(10),
                0,
                1.0,
            ),
            crate::core::framework::render::SpritePhaseInput::new(
                10,
                1,
                RenderQueueValue::GEOMETRY,
                5,
                1.0,
            ),
            crate::core::framework::render::SpritePhaseInput::new(
                20,
                2,
                RenderQueueValue::GEOMETRY,
                2,
                1.0,
            ),
        ],
    );

    assert_eq!(
        queue
            .items
            .iter()
            .map(|item| item.mesh_source)
            .collect::<Vec<_>>(),
        vec![
            RenderPhaseMeshSource::SpriteIndex(2),
            RenderPhaseMeshSource::SpriteIndex(1),
            RenderPhaseMeshSource::SpriteIndex(0),
        ]
    );
}

#[test]
fn render_product_sprite_submit_records_sprite_stats_without_particle_feature() {
    let framework =
        WgpuRenderFramework::new_for_test(Arc::new(ProjectAssetManager::default())).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("sprite-core2d-product")
                .with_pipeline_asset(RenderPipelineHandle::new(3)),
        )
        .unwrap();
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(78),
        super::render_product_submit::snapshot_with_projection_for_sprite_tests(
            ProjectionMode::Orthographic,
        ),
    );
    *extract.geometry = GeometryExtract::from_meshes(CorePipelineKind::Core2d, Vec::new());
    *extract.sprites = SpriteExtract::from_sprites(
        CorePipelineKind::Core2d,
        vec![RenderSpriteSnapshot {
            entity: 50,
            transform: Transform::default(),
            image: texture_handle("res://textures/missing-sprite.png"),
            material: None,
            atlas_region: None,
            rect: None,
            flip_x: false,
            flip_y: false,
            anchor: RenderSpriteAnchor::CENTER,
            custom_size: Some(Vec2::new(1.0, 1.0)),
            image_mode: RenderSpriteImageMode::Stretch,
            color: Vec4::ONE,
            z_order: 0,
            common: RendererCommon {
                layer_mask: RenderLayerSet::from_layers(0..u32::BITS),
                ..RendererCommon::default()
            },
            material_alpha_mode: RenderMaterialAlphaMode::Blend,
        }],
    );
    assert_eq!(
        extract
            .sprites
            .phase_queue
            .items_for_phase(RenderPhase::Transparent2d)
            .map(|item| item.mesh_source)
            .collect::<Vec<_>>(),
        vec![RenderPhaseMeshSource::SpriteIndex(0)]
    );

    framework.submit_frame_extract(viewport, extract).unwrap();

    let stats = framework.query_stats().unwrap();
    assert_eq!(stats.last_sprite_count, 1);
    assert_eq!(stats.last_sprite_ready_count, 0);
    assert_eq!(stats.last_sprite_texture_fallback_count, 1);
    assert_eq!(stats.last_sprite_graph_executed_pass_count, 3);
    assert_eq!(stats.last_sprite_image_slice_count, 1);
    assert_eq!(stats.last_sprite_expanded_image_slice_count, 0);
    assert_eq!(stats.last_particle_graph_executed_pass_count, 0);
}

fn texture_handle(label: &str) -> ResourceHandle<TextureMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}

fn material_handle(label: &str) -> ResourceHandle<MaterialMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}
