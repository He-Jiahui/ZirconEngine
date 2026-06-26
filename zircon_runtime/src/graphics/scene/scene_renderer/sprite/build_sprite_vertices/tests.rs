use crate::core::framework::render::{
    CameraRenderDescriptor, FallbackSkyboxKind, PreviewEnvironmentExtract, ProjectionMode,
    RenderFrameExtract, RenderLayerSet, RenderMaterialAlphaMode, RenderOverlayExtract,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderSpriteAnchor, RenderSpriteAtlasRegion,
    RenderSpriteImageMode, RenderSpriteScalingMode, RenderSpriteSliceBorder,
    RenderSpriteSliceScaleMode, RenderSpriteSlicer, RenderWorldSnapshotHandle, SpriteExtract,
    ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec4};
use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};
use crate::graphics::types::ViewportRenderFrame;

use super::*;

#[test]
fn build_sprite_vertices_routes_transparent3d_to_transparent3d_phase() {
    let source = include_str!("../build_sprite_vertices.rs");

    assert!(source.contains("RenderPassStage::Transparent3d => RenderPhase::Transparent3d"));
}

#[test]
fn build_sprite_vertices_filters_sprites_by_selected_camera_layers() {
    let mut hidden = test_sprite(RenderSpriteImageMode::Stretch);
    hidden.entity = 1;
    hidden.render_layer_mask = RenderLayerSet::layer(1);
    let mut visible = test_sprite(RenderSpriteImageMode::Stretch);
    visible.entity = 2;
    visible.render_layer_mask = RenderLayerSet::layer(40);

    let mut camera = ViewportCameraSnapshot::default();
    camera.projection_mode = ProjectionMode::Orthographic;
    let mut descriptor = CameraRenderDescriptor::from_camera_payload(Some(7), camera.clone());
    descriptor.culling_mask = RenderLayerSet::layer(40);
    let mut extract = empty_sprite_extract(camera, vec![hidden, visible]);
    extract.select_camera_descriptor(descriptor);
    let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(64, 64));

    let vertices = build_sprite_vertices(&frame, RenderPassStage::Transparent2d);

    assert_eq!(vertices.len(), 1);
    assert_eq!(vertices[0].0, 1);
    assert_eq!(vertices[0].1.len(), 6);
}

#[test]
fn sprite_image_vertices_keep_stretch_as_single_quad() {
    let sprite = test_sprite(RenderSpriteImageMode::Stretch);

    let vertices = sprite_image_vertices(&sprite, Vec2::new(4.0, 3.0));

    assert_eq!(vertices.len(), 6);
}

#[test]
fn sprite_image_vertices_tile_custom_size_into_repeated_quads() {
    let sprite = test_sprite(RenderSpriteImageMode::tiled(true, true, 1.0));

    let vertices = sprite_image_vertices(&sprite, Vec2::new(4.0, 3.0));

    assert_eq!(vertices.len(), 72);
}

#[test]
fn sprite_image_vertices_slice_custom_size_into_nine_regions() {
    let sprite = test_sprite(RenderSpriteImageMode::Sliced(RenderSpriteSlicer {
        border: RenderSpriteSliceBorder::all(0.25),
        center_scale_mode: RenderSpriteSliceScaleMode::Stretch,
        sides_scale_mode: RenderSpriteSliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    }));

    let vertices = sprite_image_vertices(&sprite, Vec2::new(4.0, 4.0));

    assert_eq!(vertices.len(), 54);
}

#[test]
fn sprite_image_slices_cap_excessive_tile_subdivision() {
    let slices = sprite_image_slices(
        RenderSpriteImageMode::tiled(true, true, MIN_STRETCH_VALUE),
        RenderSpriteRect::new(Vec2::ZERO, Vec2::ONE),
        Vec2::new(4_000.0, 4_000.0),
    );

    assert_eq!(slices.len(), MAX_SPRITE_IMAGE_SLICES);
}

#[test]
fn sprite_image_slices_fit_center_preserves_full_uv_and_letterboxes() {
    let base_rect = RenderSpriteRect::new(Vec2::ZERO, Vec2::new(2.0, 1.0));

    let slices = sprite_image_slices(
        RenderSpriteImageMode::scale(RenderSpriteScalingMode::FitCenter),
        base_rect,
        Vec2::new(2.0, 2.0),
    );

    assert_eq!(slices.len(), 1);
    assert_eq!(slices[0].texture_rect, base_rect);
    assert_vec2_near(slices[0].draw_size, Vec2::new(2.0, 1.0));
    assert_vec2_near(slices[0].offset, Vec2::new(1.0, 1.0));
}

#[test]
fn sprite_image_slices_fit_start_aligns_to_left_top() {
    let slices = sprite_image_slices(
        RenderSpriteImageMode::scale(RenderSpriteScalingMode::FitStart),
        RenderSpriteRect::new(Vec2::ZERO, Vec2::new(2.0, 1.0)),
        Vec2::new(2.0, 2.0),
    );

    assert_eq!(slices.len(), 1);
    assert_vec2_near(slices[0].draw_size, Vec2::new(2.0, 1.0));
    assert_vec2_near(slices[0].offset, Vec2::new(1.0, 1.5));
}

#[test]
fn sprite_image_slices_fill_center_crops_source_rect() {
    let slices = sprite_image_slices(
        RenderSpriteImageMode::scale(RenderSpriteScalingMode::FillCenter),
        RenderSpriteRect::new(Vec2::ZERO, Vec2::new(4.0, 2.0)),
        Vec2::new(2.0, 2.0),
    );

    assert_eq!(slices.len(), 1);
    assert_eq!(
        slices[0].texture_rect,
        RenderSpriteRect::new(Vec2::new(1.0, 0.0), Vec2::new(3.0, 2.0))
    );
    assert_vec2_near(slices[0].draw_size, Vec2::new(2.0, 2.0));
    assert_vec2_near(slices[0].offset, Vec2::new(1.0, 1.0));
}

#[test]
fn sprite_image_slices_fill_end_aligns_source_crop_to_right_or_bottom() {
    let horizontal = sprite_image_slices(
        RenderSpriteImageMode::scale(RenderSpriteScalingMode::FillEnd),
        RenderSpriteRect::new(Vec2::ZERO, Vec2::new(4.0, 2.0)),
        Vec2::new(2.0, 2.0),
    );
    let vertical = sprite_image_slices(
        RenderSpriteImageMode::scale(RenderSpriteScalingMode::FillEnd),
        RenderSpriteRect::new(Vec2::ZERO, Vec2::new(2.0, 4.0)),
        Vec2::new(2.0, 2.0),
    );

    assert_eq!(
        horizontal[0].texture_rect,
        RenderSpriteRect::new(Vec2::new(2.0, 0.0), Vec2::new(4.0, 2.0))
    );
    assert_eq!(
        vertical[0].texture_rect,
        RenderSpriteRect::new(Vec2::new(0.0, 0.0), Vec2::new(2.0, 2.0))
    );
}

#[test]
fn sprite_image_vertices_scale_fill_remains_single_quad() {
    let sprite = test_sprite(RenderSpriteImageMode::scale(
        RenderSpriteScalingMode::FillCenter,
    ));

    let vertices = sprite_image_vertices(&sprite, Vec2::new(4.0, 3.0));

    assert_eq!(vertices.len(), 6);
}

fn assert_vec2_near(actual: Vec2, expected: Vec2) {
    assert!((actual.x - expected.x).abs() < 0.0001, "{actual:?}");
    assert!((actual.y - expected.y).abs() < 0.0001, "{actual:?}");
}

fn test_sprite(
    image_mode: RenderSpriteImageMode,
) -> crate::core::framework::render::RenderSpriteSnapshot {
    crate::core::framework::render::RenderSpriteSnapshot {
        entity: 1,
        transform: Transform::default(),
        image: ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
            "res://textures/test-sprite.png",
        )),
        material: None,
        atlas_region: Some(RenderSpriteAtlasRegion::new(Vec2::ZERO, Vec2::ONE)),
        rect: Some(RenderSpriteRect::new(Vec2::ZERO, Vec2::ONE)),
        flip_x: false,
        flip_y: false,
        anchor: RenderSpriteAnchor::BOTTOM_LEFT,
        custom_size: None,
        image_mode,
        color: Vec4::ONE,
        z_order: 0,
        render_layer_mask: RenderLayerSet::from_layers(0..u32::BITS),
        material_alpha_mode: RenderMaterialAlphaMode::Blend,
    }
}

fn empty_sprite_extract(
    camera: ViewportCameraSnapshot,
    sprites: Vec<crate::core::framework::render::RenderSpriteSnapshot>,
) -> RenderFrameExtract {
    let core_pipeline = camera.core_pipeline_kind();
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(71),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera,
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    );
    extract.sprites = SpriteExtract::from_sprites(core_pipeline, sprites);
    extract
}
