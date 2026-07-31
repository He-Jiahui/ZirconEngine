use super::{RenderFixture, average_channel_in_region, centered_quad_transform, resource_handle};
use crate::asset::assets::AlphaMode;
use crate::core::framework::render::{
    CorePipelineKind, GeometryExtract, GeometryPhaseInput, RenderFramework, RenderLayerSet,
    RenderMaterialAlphaMode, RenderMeshSnapshot, RenderPhase, RenderQualityProfile,
    RenderSpriteAnchor, RenderSpriteImageMode, RenderSpriteSnapshot, SpriteExtract,
    SpritePhaseExtractInput,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec4};
use crate::core::resource::TextureMarker;
use crate::scene::components::{Mobility, default_render_layer_mask};

#[test]
fn transparent3d_product_interleaves_mesh_and_sprite_pixels_by_phase_sort_key() {
    let fixture = RenderFixture::new_with_alpha_mode(
        "graphics_m4_transparent_mixed_sprite_mesh",
        [1.0, 0.0, 0.0, 0.5],
        AlphaMode::Blend,
    );
    let mesh = RenderMeshSnapshot {
        node_id: 201,
        stable_instance_key: 201 << 16,
        transform_revision: 0,
        transform: centered_quad_transform(0.92),
        model: fixture.model,
        mesh: None,
        material: fixture.material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::new(1.0, 1.0, 1.0, 0.5),
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask()),
            ..Default::default()
        },
    };
    let sprite = RenderSpriteSnapshot {
        entity: 301,
        transform: Transform::default(),
        image: resource_handle::<TextureMarker>(&fixture.asset_manager, "res://textures/white.png"),
        material: None,
        atlas_region: None,
        rect: None,
        flip_x: false,
        flip_y: false,
        anchor: RenderSpriteAnchor::CENTER,
        custom_size: Some(Vec2::new(1.25, 1.25)),
        image_mode: RenderSpriteImageMode::Stretch,
        color: Vec4::new(0.0, 1.0, 0.0, 1.0),
        z_order: 0,
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask()),
            ..crate::core::framework::render::RendererCommon::default()
        },
        material_alpha_mode: RenderMaterialAlphaMode::Blend,
    };
    let mut extract = fixture.frame_extract(Vec::new(), Vec::new(), |_| {});
    extract.geometry = GeometryExtract::from_meshes_and_phase_inputs(
        CorePipelineKind::Core3d,
        vec![mesh],
        vec![GeometryPhaseInput::new(
            201,
            0,
            RenderMaterialAlphaMode::Blend,
            1.0,
        )],
    );
    extract.sprites = SpriteExtract::from_sprites_and_phase_inputs(
        CorePipelineKind::Core3d,
        vec![sprite],
        vec![SpritePhaseExtractInput::new(
            301,
            0,
            RenderMaterialAlphaMode::Blend,
            0,
            100.0,
        )],
    );
    extract.apply_viewport_size(fixture.viewport_size);
    assert_eq!(
        extract
            .geometry
            .phase_queue
            .items_for_phase(RenderPhase::Transparent3d)
            .count(),
        1
    );
    assert_eq!(
        extract
            .sprites
            .phase_queue
            .items_for_phase(RenderPhase::Transparent3d)
            .count(),
        1
    );

    let server = fixture.builtin_server();
    let frame = fixture.render_extract(
        &server,
        extract,
        RenderQualityProfile::new("transparent-mixed-sprite-mesh-product")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false)
            .with_bloom(false)
            .with_color_grading(false)
            .with_anti_alias(false),
    );
    let stats = server.query_stats().unwrap();
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&"mesh.transparent".to_string())
    );

    let sample_origin = UVec2::new(
        fixture.viewport_size.x / 2 - 12,
        fixture.viewport_size.y / 2 - 12,
    );
    let sample_size = UVec2::new(24, 24);
    let red = average_channel_in_region(&frame, sample_origin, sample_size, 0);
    let green = average_channel_in_region(&frame, sample_origin, sample_size, 1);
    let blue = average_channel_in_region(&frame, sample_origin, sample_size, 2);

    assert!(
        red > 64.0 && green > 64.0 && blue < 48.0,
        "transparent mesh should blend over the earlier transparent sprite in the final WGPU product; red={red:.2}, green={green:.2}, blue={blue:.2}"
    );
}

#[test]
fn transparent3d_product_treats_world_space_ui_sprite_as_transparent_member() {
    let fixture = RenderFixture::new_with_alpha_mode(
        "graphics_m4_world_space_ui_transparent_member",
        [1.0, 0.0, 0.0, 0.5],
        AlphaMode::Blend,
    );
    let mesh = RenderMeshSnapshot {
        node_id: 211,
        stable_instance_key: 211 << 16,
        transform_revision: 0,
        transform: centered_quad_transform(0.92),
        model: fixture.model,
        mesh: None,
        material: fixture.material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::new(1.0, 1.0, 1.0, 0.5),
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask()),
            ..Default::default()
        },
    };
    let world_space_ui_panel = RenderSpriteSnapshot {
        entity: 311,
        transform: Transform::default(),
        image: resource_handle::<TextureMarker>(&fixture.asset_manager, "res://textures/white.png"),
        material: None,
        atlas_region: None,
        rect: None,
        flip_x: false,
        flip_y: false,
        anchor: RenderSpriteAnchor::CENTER,
        custom_size: Some(Vec2::new(1.25, 1.25)),
        image_mode: RenderSpriteImageMode::Stretch,
        color: Vec4::new(0.0, 1.0, 1.0, 1.0),
        z_order: 0,
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask()),
            ..crate::core::framework::render::RendererCommon::default()
        },
        material_alpha_mode: RenderMaterialAlphaMode::Blend,
    };
    let mut extract = fixture.frame_extract(Vec::new(), Vec::new(), |_| {});
    extract.geometry = GeometryExtract::from_meshes_and_phase_inputs(
        CorePipelineKind::Core3d,
        vec![mesh],
        vec![GeometryPhaseInput::new(
            211,
            0,
            RenderMaterialAlphaMode::Blend,
            1.0,
        )],
    );
    extract.sprites = SpriteExtract::from_sprites_and_phase_inputs(
        CorePipelineKind::Core3d,
        vec![world_space_ui_panel],
        vec![
            SpritePhaseExtractInput::new(311, 0, RenderMaterialAlphaMode::Blend, 0, 100.0)
                .with_ui_z_index(3_000_000),
        ],
    );
    extract.apply_viewport_size(fixture.viewport_size);

    let mesh_key = extract
        .geometry
        .phase_queue
        .items_for_phase(RenderPhase::Transparent3d)
        .next()
        .expect("transparent mesh phase item")
        .sort_key
        .raw();
    let panel_key = extract
        .sprites
        .phase_queue
        .items_for_phase(RenderPhase::Transparent3d)
        .next()
        .expect("world-space UI sprite phase item")
        .sort_key
        .raw();
    assert!(
        panel_key < mesh_key,
        "world-space UI sprites in transparent queue must sort as 3D transparent members, not screen-space UI overlays"
    );

    let server = fixture.builtin_server();
    let frame = fixture.render_extract(
        &server,
        extract,
        RenderQualityProfile::new("world-space-ui-transparent-member-product")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false)
            .with_bloom(false)
            .with_color_grading(false)
            .with_anti_alias(false),
    );
    let stats = server.query_stats().unwrap();
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&"mesh.transparent".to_string())
    );

    let sample_origin = UVec2::new(
        fixture.viewport_size.x / 2 - 12,
        fixture.viewport_size.y / 2 - 12,
    );
    let sample_size = UVec2::new(24, 24);
    let red = average_channel_in_region(&frame, sample_origin, sample_size, 0);
    let green = average_channel_in_region(&frame, sample_origin, sample_size, 1);
    let blue = average_channel_in_region(&frame, sample_origin, sample_size, 2);

    assert!(
        red > 64.0 && green > 64.0 && blue > 64.0,
        "near transparent mesh should blend over the earlier world-space UI panel; red={red:.2}, green={green:.2}, blue={blue:.2}"
    );
}
