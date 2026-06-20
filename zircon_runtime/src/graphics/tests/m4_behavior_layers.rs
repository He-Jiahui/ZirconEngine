use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::assets::{AlphaMode, MaterialAsset, ShaderAsset, ShaderSourceLanguage};
use crate::asset::pipeline::manager::{AssetManager, ProjectAssetManager};
use crate::asset::project::{ProjectManager, ProjectManifest, ProjectPaths};
use crate::asset::{
    AssetReference, AssetUri, TextureAsset, TextureAssetDescriptor, RGBA8_UNORM_FORMAT,
    RGBA8_UNORM_SRGB_FORMAT,
};
use crate::core::framework::render::{
    CameraRenderDescriptor, CameraRenderType, CorePipelineKind, DisplayMode, FallbackSkyboxKind,
    GeometryExtract, GeometryPhaseInput, PreviewEnvironmentExtract, ProjectionMode,
    RenderBloomSettings, RenderCameraClear, RenderCameraTarget,
    RenderCameraTargetGraphImportStatus, RenderCameraTargetKind, RenderCameraTargetWritebackStatus,
    RenderCaptureSource, RenderColorGradingSettings, RenderDirectionalLightSnapshot,
    RenderFrameExtract, RenderFramework, RenderImageColorSpace, RenderImageFallbackKind,
    RenderImageUsage, RenderLayerSet, RenderMaterialAlphaMode, RenderMeshSnapshot,
    RenderOverlayExtract, RenderParticleSpriteSnapshot, RenderPhase, RenderPipelineHandle,
    RenderQualityProfile, RenderSamplerDescriptor, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderSpriteAnchor, RenderSpriteImageMode, RenderSpriteSnapshot, RenderViewportDescriptor,
    RenderViewportHandle, RenderWorldSnapshotHandle, SpriteExtract, SpritePhaseExtractInput,
    ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
    TextureMarker,
};
use crate::scene::components::{default_render_layer_mask, Mobility};
use image::{ImageBuffer, ImageFormat, Rgba};

use crate::graphics::RenderFeatureDescriptor;
use crate::graphics::{
    offline_bake_frame, OfflineBakeSettings, RenderPassExecutionContext,
    RenderPassExecutorRegistration, WgpuRenderFramework,
};

use super::plugin_render_feature_fixtures::{
    default_rendering_feature_descriptors, particle_render_feature_descriptor,
};

const GPU_SCENE_TEST_WGSL: &str =
    include_str!("../scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl");

#[test]
fn bloom_quality_profile_spreads_bright_pixels_when_enabled() {
    let fixture = RenderFixture::new("graphics_m4_bloom", [1.0, 1.0, 1.0, 1.0]);
    let extract = fixture.frame_extract(
        vec![RenderMeshSnapshot {
            node_id: 1,
            stable_instance_key: 1 << 16,
            transform_revision: 0,
            transform: centered_quad_transform(0.35),
            model: fixture.model,
            mesh: None,
            material: fixture.material,
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            render_layer_mask: default_render_layer_mask(),
        }],
        Vec::new(),
        |extract| {
            extract.post_process.bloom = RenderBloomSettings {
                threshold: 0.55,
                intensity: 1.0,
                radius: 1.0,
            };
        },
    );

    let server = fixture.server();
    let bloom_on = fixture.render_extract(
        &server,
        extract.clone(),
        RenderQualityProfile::new("bloom-on")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false),
    );
    let bloom_off = fixture.render_extract(
        &server,
        extract,
        RenderQualityProfile::new("bloom-off")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false)
            .with_bloom(false),
    );

    let bloom_ring = ring_luma(&bloom_on.rgba, fixture.viewport_size, 0.18, 0.42);
    let no_bloom_ring = ring_luma(&bloom_off.rgba, fixture.viewport_size, 0.18, 0.42);
    assert!(
        bloom_ring > no_bloom_ring + 6.0,
        "expected bloom to brighten neighboring pixels; bloom ring={bloom_ring:.2}, no-bloom ring={no_bloom_ring:.2}"
    );
}

#[test]
fn color_grading_extract_tints_scene_after_post_process() {
    let fixture = RenderFixture::new("graphics_m4_color_grading", [0.72, 0.72, 0.72, 1.0]);
    let extract = fixture.frame_extract(
        vec![RenderMeshSnapshot {
            node_id: 1,
            stable_instance_key: 1 << 16,
            transform_revision: 0,
            transform: fullscreen_quad_transform(),
            model: fixture.model,
            mesh: None,
            material: fixture.material,
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            render_layer_mask: default_render_layer_mask(),
        }],
        Vec::new(),
        |extract| {
            extract.post_process.color_grading = RenderColorGradingSettings {
                exposure: 1.18,
                contrast: 1.08,
                saturation: 0.92,
                gamma: 0.95,
                tint: Vec3::new(1.12, 0.78, 0.58),
            };
        },
    );

    let server = fixture.server();
    let graded = fixture.render_extract(
        &server,
        extract.clone(),
        RenderQualityProfile::new("grade-on")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false),
    );
    let neutral = fixture.render_extract(
        &server,
        extract,
        RenderQualityProfile::new("grade-off")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false)
            .with_color_grading(false),
    );

    let graded_red = average_channel(&graded.rgba, 0);
    let graded_blue = average_channel(&graded.rgba, 2);
    let neutral_red = average_channel(&neutral.rgba, 0);
    let neutral_blue = average_channel(&neutral.rgba, 2);
    assert!(
        (graded_red - graded_blue) > (neutral_red - neutral_blue) + 12.0,
        "expected color grading tint to bias warm channels; graded delta={:.2}, neutral delta={:.2}",
        graded_red - graded_blue,
        neutral_red - neutral_blue
    );
}

#[test]
fn offline_bake_outputs_baked_lighting_and_reflection_probe_data_that_changes_rendering() {
    let fixture = RenderFixture::new("graphics_m4_offline_bake", [0.5, 0.5, 0.5, 1.0]);
    let base_extract = fixture.frame_extract(
        vec![RenderMeshSnapshot {
            node_id: 1,
            stable_instance_key: 1 << 16,
            transform_revision: 0,
            transform: fullscreen_quad_transform(),
            model: fixture.model,
            mesh: None,
            material: fixture.material,
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            render_layer_mask: default_render_layer_mask(),
        }],
        vec![RenderDirectionalLightSnapshot {
            node_id: 7,
            light_id: 7,
            layer_mask: default_render_layer_mask(),
            direction: Vec3::new(-0.4, -0.4, -1.0).normalize_or_zero(),
            color: Vec3::new(1.0, 0.62, 0.28),
            intensity: 3.2,
            shadow: None,
        }],
        |_extract| {},
    );

    let bake_output = offline_bake_frame(
        &base_extract,
        &OfflineBakeSettings {
            ambient_scale: 0.24,
            reflection_probe_scale: 0.8,
            max_reflection_probes: 1,
        },
    );
    assert!(
        bake_output.baked_lighting.intensity > 0.0,
        "offline bake should produce non-zero baked lighting"
    );
    assert!(
        !bake_output.reflection_probes.is_empty(),
        "offline bake should produce at least one reflection probe"
    );

    let mut baked_extract = base_extract.clone();
    baked_extract.lighting.baked_lighting = Some(bake_output.baked_lighting);
    baked_extract.lighting.reflection_probes = bake_output.reflection_probes;

    let server = fixture.server();
    let baked_frame = fixture.render_extract(
        &server,
        baked_extract,
        RenderQualityProfile::new("baked-on")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false),
    );
    let unbaked_frame = fixture.render_extract(
        &server,
        base_extract,
        RenderQualityProfile::new("baked-off")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false)
            .with_baked_lighting(false)
            .with_reflection_probes(false),
    );

    let baked_red = average_channel(&baked_frame.rgba, 0);
    let unbaked_red = average_channel(&unbaked_frame.rgba, 0);
    assert!(
        baked_red > unbaked_red + 8.0,
        "expected baked lighting and probes to change the frame; baked red={baked_red:.2}, unbaked red={unbaked_red:.2}"
    );
}

#[test]
fn particle_rendering_draws_billboard_sprites_in_transparent_stage() {
    let fixture = RenderFixture::new("graphics_m4_particles", [0.1, 0.1, 0.1, 1.0]);
    let extract = fixture.frame_extract(Vec::new(), Vec::new(), |extract| {
        extract.particles.emitters = vec![42];
        extract.particles.sprites = vec![RenderParticleSpriteSnapshot {
            entity: 42,
            stable_sprite_key: 0,
            position: Vec3::ZERO,
            size: 0.9,
            aspect_ratio: 1.0,
            billboard_offset: crate::core::math::Vec2::ZERO,
            rotation: 0.0,
            sort_order: 0,
            color: Vec4::new(1.0, 0.48, 0.12, 0.8),
            intensity: 1.0,
            depth_test: true,
            material: None,
            texture: None,
        }];
    });

    let particle_server = fixture.server_with_render_features(
        [particle_render_feature_descriptor()],
        [RenderPassExecutorRegistration::new(
            "particle.transparent",
            particle_transparent_billboard_executor,
        )],
    );
    let particle_frame = fixture.render_extract(
        &particle_server,
        extract.clone(),
        RenderQualityProfile::new("particle-on")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false),
    );
    let no_particle_frame = fixture.render_extract(
        &particle_server,
        extract,
        RenderQualityProfile::new("particle-off")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false)
            .with_particle_rendering(false),
    );

    let particle_pixels = warm_pixels(&particle_frame.rgba);
    let no_particle_pixels = warm_pixels(&no_particle_frame.rgba);
    assert!(
        particle_pixels > no_particle_pixels + 96,
        "expected particle rendering to add visible billboard pixels; particle={particle_pixels}, disabled={no_particle_pixels}"
    );
}

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
        render_layer_mask: default_render_layer_mask(),
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
        render_layer_mask: default_render_layer_mask(),
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
    assert!(stats
        .last_graph_executed_executor_ids
        .contains(&"mesh.transparent".to_string()));

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
        render_layer_mask: default_render_layer_mask(),
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
        render_layer_mask: default_render_layer_mask(),
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
    assert!(stats
        .last_graph_executed_executor_ids
        .contains(&"mesh.transparent".to_string()));

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

#[test]
fn primary_surface_base_camera_render_order_swap_changes_composite() {
    let fixture = RenderFixture::new("graphics_m4_primary_base_order", [1.0, 1.0, 1.0, 1.0]);
    let green_last = render_primary_base_order_scene(&fixture, 0, 1);
    let red_last = render_primary_base_order_scene(&fixture, 1, 0);
    let pixel_count = (fixture.viewport_size.x * fixture.viewport_size.y) as usize;

    let green_last_green = dominant_green_pixels(&green_last.rgba);
    let green_last_red = dominant_red_pixels(&green_last.rgba);
    assert!(
        green_last_green > pixel_count * 3 / 4 && green_last_red < pixel_count / 10,
        "later green Base camera should own the final primary surface composite; green={green_last_green}, red={green_last_red}, total={pixel_count}"
    );

    let red_last_red = dominant_red_pixels(&red_last.rgba);
    let red_last_green = dominant_green_pixels(&red_last.rgba);
    assert!(
        red_last_red > pixel_count * 3 / 4 && red_last_green < pixel_count / 10,
        "later red Base camera should own the final primary surface composite after render_order swap; red={red_last_red}, green={red_last_green}, total={pixel_count}"
    );
}

#[test]
fn primary_surface_overlay_clear_depth_controls_depth_reuse() {
    let fixture = RenderFixture::new(
        "graphics_m4_primary_overlay_clear_depth",
        [1.0, 1.0, 1.0, 1.0],
    );
    let depth_loaded = render_primary_overlay_depth_scene(&fixture, false);
    let depth_cleared = render_primary_overlay_depth_scene(&fixture, true);
    let sample_origin = UVec2::new(
        fixture.viewport_size.x / 2 - 12,
        fixture.viewport_size.y / 2 - 12,
    );
    let sample_size = UVec2::new(24, 24);

    let loaded_red = average_channel_in_region(&depth_loaded, sample_origin, sample_size, 0);
    let loaded_green = average_channel_in_region(&depth_loaded, sample_origin, sample_size, 1);
    assert!(
        loaded_red > 128.0 && loaded_green < 64.0,
        "Overlay clear_depth=false should preserve Base depth and keep the farther green overlay behind the red Base mesh; red={loaded_red:.2}, green={loaded_green:.2}"
    );

    let cleared_red = average_channel_in_region(&depth_cleared, sample_origin, sample_size, 0);
    let cleared_green = average_channel_in_region(&depth_cleared, sample_origin, sample_size, 1);
    assert!(
        cleared_green > 128.0 && cleared_red < 64.0,
        "Overlay clear_depth=true should clear Base depth and let the farther green overlay replace the center pixels; red={cleared_red:.2}, green={cleared_green:.2}"
    );
}

#[test]
fn texture_target_overlay_camera_draws_layered_mesh_over_base_clear() {
    let fixture = RenderFixture::new("graphics_m4_texture_overlay_stack", [0.0, 0.86, 0.12, 1.0]);
    let texture_id = fixture.insert_srgb_render_target_texture(
        "res://tests/camera-target/overlay-layered-composite.texture",
        fixture.viewport_size,
    );
    let overlay_layer = 2;
    let mut extract = fixture.frame_extract(
        vec![RenderMeshSnapshot {
            node_id: 101,
            stable_instance_key: 101 << 16,
            transform_revision: 0,
            transform: centered_quad_transform(0.75),
            model: fixture.model,
            mesh: None,
            material: fixture.material,
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            render_layer_mask: 1u32 << overlay_layer,
        }],
        Vec::new(),
        |_| {},
    );
    let base_camera = texture_camera_descriptor(
        1,
        texture_id,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(1),
        extract.view.camera.clone(),
    )
    .with_stack([2]);
    let overlay_camera = texture_camera_descriptor(
        2,
        texture_id,
        CameraRenderType::Overlay,
        RenderCameraClear::None,
        false,
        RenderLayerSet::layer(overlay_layer),
        extract.view.camera.clone(),
    );
    extract.view = extract.view.with_cameras(vec![base_camera, overlay_camera]);

    let server = fixture.builtin_server();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(fixture.viewport_size))
        .unwrap();
    server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(1))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("texture-overlay-stack")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_anti_alias(false),
        )
        .unwrap();

    server.submit_frame_extract(viewport, extract).unwrap();
    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("texture target overlay frame should be capturable");
    let stats = server.query_stats().unwrap();
    server.destroy_viewport(viewport).unwrap();

    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::TextureDirectGraphImport
    );
    assert_eq!(
        frame.capture_report.graph_import_status,
        RenderCameraTargetGraphImportStatus::DirectImported
    );
    assert_eq!(
        frame.capture_report.writeback_status,
        RenderCameraTargetWritebackStatus::SkippedDirectImport
    );
    assert_eq!(
        stats.last_camera_target_graph_import.status,
        RenderCameraTargetGraphImportStatus::DirectImported
    );
    let pixel_count = (frame.width * frame.height) as usize;
    let red_pixels = dominant_red_pixels(&frame.rgba);
    let green_pixels = dominant_green_pixels(&frame.rgba);
    assert!(
        red_pixels > pixel_count / 3,
        "base red clear should remain visible around overlay quad; red={red_pixels}, total={pixel_count}"
    );
    assert!(
        green_pixels > pixel_count / 20,
        "overlay-only green quad should draw over the loaded base target; green={green_pixels}, total={pixel_count}"
    );
}

#[test]
fn texture_target_overlay_camera_converts_linear_final_product_after_composite() {
    let fixture = RenderFixture::new(
        "graphics_m4_texture_overlay_linear_product",
        [0.0, 0.86, 0.12, 1.0],
    );
    let texture_id = fixture.insert_linear_render_target_texture(
        "res://tests/camera-target/overlay-layered-linear-product.texture",
        fixture.viewport_size,
    );
    let overlay_layer = 2;
    let mut extract = fixture.frame_extract(
        vec![RenderMeshSnapshot {
            node_id: 102,
            stable_instance_key: 102 << 16,
            transform_revision: 0,
            transform: centered_quad_transform(0.75),
            model: fixture.model,
            mesh: None,
            material: fixture.material,
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            render_layer_mask: 1u32 << overlay_layer,
        }],
        Vec::new(),
        |_| {},
    );
    let base_camera = texture_camera_descriptor(
        1,
        texture_id,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(1),
        extract.view.camera.clone(),
    )
    .with_stack([2]);
    let overlay_camera = texture_camera_descriptor(
        2,
        texture_id,
        CameraRenderType::Overlay,
        RenderCameraClear::None,
        false,
        RenderLayerSet::layer(overlay_layer),
        extract.view.camera.clone(),
    );
    extract.view = extract.view.with_cameras(vec![base_camera, overlay_camera]);

    let server = fixture.builtin_server();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(fixture.viewport_size))
        .unwrap();
    server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(1))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("texture-overlay-linear-product")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_anti_alias(false),
        )
        .unwrap();

    server.submit_frame_extract(viewport, extract).unwrap();
    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("linear texture target overlay frame should be capturable");
    let stats = server.query_stats().unwrap();
    let (texture_size, texture_rgba) = server
        .read_output_target_texture_rgba_for_tests(texture_id)
        .unwrap()
        .expect("linear texture target product should remain readable after writeback");
    server.destroy_viewport(viewport).unwrap();

    assert_eq!(texture_size, fixture.viewport_size);
    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::Texture
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::TextureWritebackConversion
    );
    assert_eq!(
        frame.capture_report.graph_import_status,
        RenderCameraTargetGraphImportStatus::RequiresConversionWriteback
    );
    assert_eq!(
        frame.capture_report.writeback_status,
        RenderCameraTargetWritebackStatus::Converted
    );
    assert_eq!(
        stats.last_camera_target_graph_import.status,
        RenderCameraTargetGraphImportStatus::RequiresConversionWriteback
    );
    assert_eq!(
        stats
            .last_camera_target_graph_import
            .conversion_writeback_count,
        1
    );
    assert_eq!(
        stats.last_camera_target_writeback.status,
        RenderCameraTargetWritebackStatus::Converted
    );
    assert_eq!(stats.last_camera_target_writeback.converted_count, 1);
    assert_eq!(stats.last_camera_target_writeback.copied_count, 0);

    let pixel_count = (frame.width * frame.height) as usize;
    let frame_red_pixels = dominant_red_pixels(&frame.rgba);
    let frame_green_pixels = dominant_green_pixels(&frame.rgba);
    let texture_red_pixels = dominant_red_pixels(&texture_rgba);
    let texture_green_pixels = dominant_green_pixels(&texture_rgba);
    assert!(
        frame_red_pixels > pixel_count / 3,
        "base red clear should remain visible in converted final product; red={frame_red_pixels}, total={pixel_count}"
    );
    assert!(
        frame_green_pixels > pixel_count / 20,
        "overlay green quad should draw into converted final product; green={frame_green_pixels}, total={pixel_count}"
    );
    assert!(
        texture_red_pixels > pixel_count / 3,
        "converted texture target should keep the base composite; red={texture_red_pixels}, total={pixel_count}"
    );
    assert!(
        texture_green_pixels > pixel_count / 20,
        "converted texture target should include the terminal overlay draw; green={texture_green_pixels}, total={pixel_count}"
    );
}

#[test]
fn texture_target_stack_preserves_composite_when_primary_surface_renders_later() {
    let fixture = RenderFixture::new(
        "graphics_m4_texture_stack_then_primary_surface",
        [0.0, 0.86, 0.12, 1.0],
    );
    let texture_id = fixture.insert_srgb_render_target_texture(
        "res://tests/camera-target/stack-before-primary.texture",
        fixture.viewport_size,
    );
    let overlay_layer = 2;
    let mut extract = fixture.frame_extract(
        vec![RenderMeshSnapshot {
            node_id: 103,
            stable_instance_key: 103 << 16,
            transform_revision: 0,
            transform: centered_quad_transform(0.75),
            model: fixture.model,
            mesh: None,
            material: fixture.material,
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            render_layer_mask: 1u32 << overlay_layer,
        }],
        Vec::new(),
        |_| {},
    );
    let texture_base = texture_camera_descriptor(
        1,
        texture_id,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(1),
        extract.view.camera.clone(),
    )
    .with_stack([2]);
    let texture_overlay = texture_camera_descriptor(
        2,
        texture_id,
        CameraRenderType::Overlay,
        RenderCameraClear::None,
        false,
        RenderLayerSet::layer(overlay_layer),
        extract.view.camera.clone(),
    );
    let primary = primary_surface_camera_descriptor(
        3,
        1,
        RenderCameraClear::Color(Vec4::new(0.0, 0.0, 1.0, 1.0)),
        RenderLayerSet::layer(0),
        extract.view.camera.clone(),
    );
    extract.view = extract
        .view
        .with_cameras(vec![texture_base, texture_overlay, primary]);

    let server = fixture.builtin_server();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(fixture.viewport_size))
        .unwrap();
    server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(1))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("texture-stack-before-primary-product")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_anti_alias(false),
        )
        .unwrap();

    server.submit_frame_extract(viewport, extract).unwrap();
    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("primary surface terminal frame should be capturable");
    let (texture_size, texture_rgba) = server
        .read_output_target_texture_rgba_for_tests(texture_id)
        .unwrap()
        .expect("texture target stack should remain prepared after primary surface submit");
    server.destroy_viewport(viewport).unwrap();

    assert_eq!(texture_size, fixture.viewport_size);
    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::PrimarySurface
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::FrameworkOffscreen
    );

    let pixel_count = (frame.width * frame.height) as usize;
    let frame_blue_pixels = dominant_blue_pixels(&frame.rgba);
    let frame_red_pixels = dominant_red_pixels(&frame.rgba);
    let frame_green_pixels = dominant_green_pixels(&frame.rgba);
    assert!(
        frame_blue_pixels > pixel_count * 3 / 4
            && frame_red_pixels < pixel_count / 10
            && frame_green_pixels < pixel_count / 10,
        "later PrimarySurface Base should own the viewport capture without overwriting the earlier texture target; blue={frame_blue_pixels}, red={frame_red_pixels}, green={frame_green_pixels}, total={pixel_count}"
    );

    let texture_red_pixels = dominant_red_pixels(&texture_rgba);
    let texture_green_pixels = dominant_green_pixels(&texture_rgba);
    let texture_blue_pixels = dominant_blue_pixels(&texture_rgba);
    assert!(
        texture_red_pixels > pixel_count / 3,
        "texture target stack should keep the red Base clear after a later PrimarySurface camera; red={texture_red_pixels}, blue={texture_blue_pixels}, total={pixel_count}"
    );
    assert!(
        texture_green_pixels > pixel_count / 20,
        "texture target stack should keep the green Overlay mesh after a later PrimarySurface camera; green={texture_green_pixels}, total={pixel_count}"
    );
    assert!(
        texture_blue_pixels < pixel_count / 10,
        "later PrimarySurface blue clear must not leak into the custom texture target; blue={texture_blue_pixels}, total={pixel_count}"
    );
}

#[test]
fn texture_target_render_order_feeds_later_primary_surface_material_sample() {
    let fixture = RenderFixture::new(
        "graphics_m4_texture_target_sampled_later",
        [1.0, 1.0, 1.0, 1.0],
    );
    let texture_uri = "res://tests/camera-target/sample-source.texture";
    let texture_id = fixture.insert_srgb_render_target_texture(texture_uri, fixture.viewport_size);
    let sampled_material = fixture.insert_texture_sampling_material(
        "res://materials/sample-output-target.zmaterial",
        texture_uri,
    );
    let mut extract = fixture.frame_extract(
        vec![RenderMeshSnapshot {
            node_id: 121,
            stable_instance_key: 121 << 16,
            transform_revision: 0,
            transform: centered_quad_transform(0.82),
            model: fixture.model,
            mesh: None,
            material: sampled_material,
            mesh_lod: None,
            morph_weights: Vec::new(),
            tint: Vec4::ONE,
            mobility: Mobility::Dynamic,
            static_state: Default::default(),
            render_layer_mask: default_render_layer_mask(),
        }],
        Vec::new(),
        |_| {},
    );
    let texture_camera = texture_camera_descriptor(
        1,
        texture_id,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        true,
        RenderLayerSet::layer(7),
        extract.view.camera.clone(),
    );
    let primary = primary_surface_camera_descriptor(
        2,
        1,
        RenderCameraClear::Color(Vec4::new(0.0, 0.0, 1.0, 1.0)),
        RenderLayerSet::default(),
        extract.view.camera.clone(),
    );
    extract.view = extract.view.with_cameras(vec![texture_camera, primary]);

    let server = fixture.builtin_server();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(fixture.viewport_size))
        .unwrap();
    server
        .set_pipeline_asset(viewport, RenderPipelineHandle::new(1))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("texture-target-sampled-by-later-primary")
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_anti_alias(false),
        )
        .unwrap();

    server.submit_frame_extract(viewport, extract).unwrap();
    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("primary surface terminal frame should be capturable");
    let (texture_size, texture_rgba) = server
        .read_output_target_texture_rgba_for_tests(texture_id)
        .unwrap()
        .expect("sample source texture target should remain prepared after primary surface submit");
    server.destroy_viewport(viewport).unwrap();

    assert_eq!(texture_size, fixture.viewport_size);
    assert_eq!(
        frame.capture_report.target_kind,
        RenderCameraTargetKind::PrimarySurface
    );
    assert_eq!(
        frame.capture_report.source,
        RenderCaptureSource::FrameworkOffscreen
    );

    let pixel_count = (frame.width * frame.height) as usize;
    let texture_red_pixels = dominant_red_pixels(&texture_rgba);
    assert!(
        texture_red_pixels > pixel_count * 3 / 4,
        "source texture target should be red before the later PrimarySurface samples it; red={texture_red_pixels}, total={pixel_count}"
    );

    let frame_red_pixels = dominant_red_pixels(&frame.rgba);
    let frame_blue_pixels = dominant_blue_pixels(&frame.rgba);
    assert!(
        frame_red_pixels > pixel_count / 20 && frame_blue_pixels > pixel_count / 3,
        "later PrimarySurface should show a red mesh sampled from the earlier texture target over its blue clear; red={frame_red_pixels}, blue={frame_blue_pixels}, total={pixel_count}"
    );
}

#[test]
fn particle_shader_preserves_sprite_alpha_for_transparent_blending() {
    let shader = include_str!("../scene/scene_renderer/particle/shaders/particle.wgsl");

    assert!(
        shader.contains("return input.color;"),
        "particle shader should preserve vertex alpha instead of forcing opaque output"
    );
}

struct RenderFixture {
    root: PathBuf,
    asset_manager: Arc<ProjectAssetManager>,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    viewport_size: UVec2,
}

impl RenderFixture {
    fn new(label: &str, base_color: [f32; 4]) -> Self {
        Self::new_with_alpha_mode(label, base_color, AlphaMode::Opaque)
    }

    fn new_with_alpha_mode(label: &str, base_color: [f32; 4], alpha_mode: AlphaMode) -> Self {
        let root = unique_temp_project_root(label);
        let paths = ProjectPaths::from_root(&root).unwrap();
        paths.ensure_layout().unwrap();
        ProjectManifest::new(
            label,
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        )
        .save(paths.manifest_path())
        .unwrap();

        write_flat_color_wgsl(
            paths.assets_root().join("shaders").join("flat_color.wgsl"),
            [base_color[0], base_color[1], base_color[2]],
        );
        write_solid_png(
            paths.assets_root().join("textures").join("white.png"),
            [255, 255, 255, 255],
        );
        write_quad_obj(paths.assets_root().join("models").join("quad.obj"));
        write_material_with_base_color_and_texture(
            paths
                .assets_root()
                .join("materials")
                .join("flat_color.zmaterial"),
            "res://shaders/flat_color.wgsl",
            base_color,
            "res://textures/white.png",
            alpha_mode,
        );

        let asset_manager = Arc::new(ProjectAssetManager::default());
        asset_manager
            .register_first_wave_plugin_fixture_importers_for_test()
            .unwrap();
        asset_manager
            .open_project(root.to_string_lossy().as_ref())
            .unwrap();
        let mut project = ProjectManager::open(&root).unwrap();
        project.scan_and_import().unwrap();

        let model = resource_handle::<ModelMarker>(&asset_manager, "res://models/quad.obj");
        let material = resource_handle::<MaterialMarker>(
            &asset_manager,
            "res://materials/flat_color.zmaterial",
        );

        Self {
            root,
            asset_manager,
            model,
            material,
            viewport_size: UVec2::new(160, 120),
        }
    }

    fn server(&self) -> WgpuRenderFramework {
        WgpuRenderFramework::new_with_plugin_render_features(
            self.asset_manager.clone(),
            default_rendering_feature_descriptors(),
            Vec::new(),
            Vec::new(),
        )
        .unwrap()
    }

    fn builtin_server(&self) -> WgpuRenderFramework {
        WgpuRenderFramework::new(self.asset_manager.clone()).unwrap()
    }

    fn server_with_render_features(
        &self,
        render_features: impl IntoIterator<Item = RenderFeatureDescriptor>,
        render_pass_executors: impl IntoIterator<Item = RenderPassExecutorRegistration>,
    ) -> WgpuRenderFramework {
        let mut features = default_rendering_feature_descriptors();
        features.extend(render_features);
        WgpuRenderFramework::new_with_plugin_render_features(
            self.asset_manager.clone(),
            features,
            render_pass_executors,
            Vec::new(),
        )
        .unwrap()
    }

    fn frame_extract<F>(
        &self,
        meshes: Vec<RenderMeshSnapshot>,
        lights: Vec<RenderDirectionalLightSnapshot>,
        configure: F,
    ) -> RenderFrameExtract
    where
        F: FnOnce(&mut RenderFrameExtract),
    {
        let mut extract = RenderFrameExtract::from_snapshot(
            RenderWorldSnapshotHandle::new(1),
            build_snapshot(meshes, lights, self.viewport_size),
        );
        configure(&mut extract);
        extract
    }

    fn insert_linear_render_target_texture(&self, uri: &str, size: UVec2) -> ResourceId {
        self.insert_render_target_texture(uri, size, render_target_texture_descriptor())
    }

    fn insert_srgb_render_target_texture(&self, uri: &str, size: UVec2) -> ResourceId {
        self.insert_render_target_texture(uri, size, srgb_render_target_texture_descriptor())
    }

    fn insert_render_target_texture(
        &self,
        uri: &str,
        size: UVec2,
        descriptor: TextureAssetDescriptor,
    ) -> ResourceId {
        let texture_uri = AssetUri::parse(uri).unwrap();
        let texture_id = ResourceId::from_locator(&texture_uri);
        self.asset_manager
            .assets::<TextureAsset>()
            .insert(
                ResourceRecord::new(texture_id, ResourceKind::Texture, texture_uri.clone()),
                TextureAsset::new_rgba8(
                    texture_uri,
                    size.x,
                    size.y,
                    vec![0; (size.x * size.y * 4) as usize],
                )
                .with_descriptor(descriptor),
            )
            .expect("texture insert");
        texture_id
    }

    fn insert_texture_sampling_material(
        &self,
        material_uri: &str,
        base_color_texture_uri: &str,
    ) -> ResourceHandle<MaterialMarker> {
        let shader_uri = AssetUri::parse("res://shaders/sample_texture.wgsl").unwrap();
        let shader_id = ResourceId::from_locator(&shader_uri);
        self.asset_manager
            .assets::<ShaderAsset>()
            .insert(
                ResourceRecord::new(shader_id, ResourceKind::Shader, shader_uri.clone()),
                sample_texture_shader(shader_uri),
            )
            .expect("sample texture shader insert");

        let material_uri = AssetUri::parse(material_uri).unwrap();
        let material_id = ResourceId::from_locator(&material_uri);
        self.asset_manager
            .assets::<MaterialAsset>()
            .insert(
                ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
                MaterialAsset {
                    name: Some("SampleOutputTarget".to_string()),
                    shader: asset_reference("res://shaders/sample_texture.wgsl"),
                    base_color: [1.0, 1.0, 1.0, 1.0],
                    base_color_texture: Some(asset_reference(base_color_texture_uri)),
                    normal_texture: None,
                    metallic: 0.0,
                    roughness: 1.0,
                    metallic_roughness_texture: None,
                    occlusion_texture: None,
                    emissive: [0.0, 0.0, 0.0],
                    emissive_texture: None,
                    alpha_mode: AlphaMode::Opaque,
                    double_sided: false,
                    property_values: Default::default(),
                    texture_slots: Default::default(),
                    validation_diagnostics: Vec::new(),
                },
            )
            .expect("sample texture material insert");
        ResourceHandle::new(material_id)
    }

    fn render_extract(
        &self,
        server: &WgpuRenderFramework,
        extract: RenderFrameExtract,
        profile: RenderQualityProfile,
    ) -> crate::core::framework::render::CapturedFrame {
        let viewport = server
            .create_viewport(RenderViewportDescriptor::new(self.viewport_size))
            .unwrap();
        server
            .set_pipeline_asset(viewport, RenderPipelineHandle::new(1))
            .unwrap();
        server.set_quality_profile(viewport, profile).unwrap();
        let frame = submit_extract(server, viewport, extract);
        server.destroy_viewport(viewport).unwrap();
        frame
    }
}

fn particle_transparent_billboard_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    context
        .require_gpu()?
        .record_particle_billboards_to_resources("scene-color", "scene-depth")
}

impl Drop for RenderFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn unique_temp_project_root(label: &str) -> PathBuf {
    static NEXT_TEMP_PROJECT_ID: AtomicU64 = AtomicU64::new(1);

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let process_id = std::process::id();
    let sequence = NEXT_TEMP_PROJECT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "zircon_graphics_{label}_{process_id}_{sequence}_{unique}"
    ))
}

fn build_snapshot(
    meshes: Vec<RenderMeshSnapshot>,
    lights: Vec<RenderDirectionalLightSnapshot>,
    viewport_size: UVec2,
) -> RenderSceneSnapshot {
    let mut camera = ViewportCameraSnapshot {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 4.0),
            ..Transform::default()
        },
        projection_mode: ProjectionMode::Perspective,
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(viewport_size);

    RenderSceneSnapshot {
        scene: RenderSceneGeometryExtract {
            camera,
            meshes,
            directional_lights: lights,
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_lights: Vec::new(),
            rect_lights: Vec::new(),
        },
        overlays: RenderOverlayExtract {
            display_mode: DisplayMode::Shaded,
            ..RenderOverlayExtract::default()
        },
        preview: PreviewEnvironmentExtract {
            lighting_enabled: false,
            skybox_enabled: false,
            fallback_skybox: FallbackSkyboxKind::None,
            clear_color: Vec4::ZERO,
        },
        virtual_geometry_debug: None,
    }
}

fn texture_camera_descriptor(
    entity: u64,
    texture_id: ResourceId,
    render_type: CameraRenderType,
    clear: RenderCameraClear,
    clear_depth: bool,
    layers: RenderLayerSet,
    camera: ViewportCameraSnapshot,
) -> CameraRenderDescriptor {
    CameraRenderDescriptor {
        entity: Some(entity),
        render_type,
        target: RenderCameraTarget::Texture(ResourceHandle::<TextureMarker>::new(texture_id)),
        clear,
        clear_depth,
        culling_mask: layers.clone(),
        volume_mask: layers,
        ..CameraRenderDescriptor::from_camera_payload(Some(entity), camera)
    }
}

fn primary_surface_camera_descriptor(
    entity: u64,
    render_order: i32,
    clear: RenderCameraClear,
    layers: RenderLayerSet,
    camera: ViewportCameraSnapshot,
) -> CameraRenderDescriptor {
    CameraRenderDescriptor {
        entity: Some(entity),
        render_order,
        render_type: CameraRenderType::Base,
        target: RenderCameraTarget::default(),
        clear,
        culling_mask: layers.clone(),
        volume_mask: layers,
        ..CameraRenderDescriptor::from_camera_payload(Some(entity), camera)
    }
}

fn primary_surface_camera_stack_descriptor(
    entity: u64,
    render_type: CameraRenderType,
    clear: RenderCameraClear,
    clear_depth: bool,
    layers: RenderLayerSet,
    camera: ViewportCameraSnapshot,
) -> CameraRenderDescriptor {
    CameraRenderDescriptor {
        entity: Some(entity),
        render_type,
        target: RenderCameraTarget::default(),
        clear,
        clear_depth,
        culling_mask: layers.clone(),
        volume_mask: layers,
        ..CameraRenderDescriptor::from_camera_payload(Some(entity), camera)
    }
}

fn render_primary_base_order_scene(
    fixture: &RenderFixture,
    red_order: i32,
    green_order: i32,
) -> crate::core::framework::render::CapturedFrame {
    let mut extract = fixture.frame_extract(Vec::new(), Vec::new(), |_| {});
    let red_camera = primary_surface_camera_descriptor(
        401,
        red_order,
        RenderCameraClear::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
        RenderLayerSet::default(),
        extract.view.camera.clone(),
    );
    let green_camera = primary_surface_camera_descriptor(
        402,
        green_order,
        RenderCameraClear::Color(Vec4::new(0.0, 1.0, 0.0, 1.0)),
        RenderLayerSet::default(),
        extract.view.camera.clone(),
    );
    extract.view = extract.view.with_cameras(vec![red_camera, green_camera]);

    let server = fixture.builtin_server();
    fixture.render_extract(
        &server,
        extract,
        RenderQualityProfile::new("primary-base-render-order-product")
            .with_clustered_lighting(false)
            .with_screen_space_ambient_occlusion(false)
            .with_temporal_history(false)
            .with_bloom(false)
            .with_color_grading(false)
            .with_anti_alias(false),
    )
}

fn render_primary_overlay_depth_scene(
    fixture: &RenderFixture,
    overlay_clear_depth: bool,
) -> crate::core::framework::render::CapturedFrame {
    let base_layer = 1;
    let overlay_layer = 2;
    let mut extract = fixture.frame_extract(
        vec![
            RenderMeshSnapshot {
                node_id: 501,
                stable_instance_key: 501 << 16,
                transform_revision: 0,
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 1.0),
                    scale: Vec3::new(1.1, 1.1, 1.0),
                    ..Transform::default()
                },
                model: fixture.model,
                mesh: None,
                material: fixture.material,
                mesh_lod: None,
                morph_weights: Vec::new(),
                tint: Vec4::new(1.0, 0.0, 0.0, 1.0),
                mobility: Mobility::Dynamic,
                static_state: Default::default(),
                render_layer_mask: 1u32 << base_layer,
            },
            RenderMeshSnapshot {
                node_id: 502,
                stable_instance_key: 502 << 16,
                transform_revision: 0,
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    scale: Vec3::new(0.82, 0.82, 1.0),
                    ..Transform::default()
                },
                model: fixture.model,
                mesh: None,
                material: fixture.material,
                mesh_lod: None,
                morph_weights: Vec::new(),
                tint: Vec4::new(0.0, 1.0, 0.0, 1.0),
                mobility: Mobility::Dynamic,
                static_state: Default::default(),
                render_layer_mask: 1u32 << overlay_layer,
            },
        ],
        Vec::new(),
        |_| {},
    );
    let base_camera = primary_surface_camera_stack_descriptor(
        501,
        CameraRenderType::Base,
        RenderCameraClear::Color(Vec4::new(0.0, 0.0, 1.0, 1.0)),
        true,
        RenderLayerSet::layer(base_layer),
        extract.view.camera.clone(),
    )
    .with_stack([502]);
    let overlay_camera = primary_surface_camera_stack_descriptor(
        502,
        CameraRenderType::Overlay,
        RenderCameraClear::None,
        overlay_clear_depth,
        RenderLayerSet::layer(overlay_layer),
        extract.view.camera.clone(),
    );
    extract.view = extract.view.with_cameras(vec![base_camera, overlay_camera]);

    let server = fixture.builtin_server();
    fixture.render_extract(
        &server,
        extract,
        RenderQualityProfile::new(if overlay_clear_depth {
            "primary-overlay-clear-depth"
        } else {
            "primary-overlay-load-depth"
        })
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false),
    )
}

fn fullscreen_quad_transform() -> Transform {
    Transform {
        scale: Vec3::new(1.8, 1.8, 1.0),
        ..Transform::default()
    }
}

fn centered_quad_transform(scale: f32) -> Transform {
    Transform {
        scale: Vec3::new(scale, scale, 1.0),
        ..Transform::default()
    }
}

fn submit_extract(
    server: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
) -> crate::core::framework::render::CapturedFrame {
    server.submit_frame_extract(viewport, extract).unwrap();
    server
        .capture_frame(viewport)
        .unwrap()
        .expect("frame should be available after submission")
}

fn ring_luma(rgba: &[u8], viewport_size: UVec2, inner_radius: f32, outer_radius: f32) -> f32 {
    let mut total = 0.0;
    let mut count = 0.0;
    let center_x = viewport_size.x as f32 * 0.5;
    let center_y = viewport_size.y as f32 * 0.5;
    let normalizer = viewport_size.x.min(viewport_size.y) as f32 * 0.5;
    for y in 0..viewport_size.y as usize {
        for x in 0..viewport_size.x as usize {
            let dx = x as f32 + 0.5 - center_x;
            let dy = y as f32 + 0.5 - center_y;
            let radius = (dx * dx + dy * dy).sqrt() / normalizer.max(1.0);
            if radius < inner_radius || radius > outer_radius {
                continue;
            }
            let index = (y * viewport_size.x as usize + x) * 4;
            let pixel = &rgba[index..index + 4];
            total += 0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32;
            count += 1.0;
        }
    }
    if count <= 0.0 {
        0.0
    } else {
        total / count
    }
}

fn warm_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| {
            // Particle sprites preserve transparent alpha, so visible warm pixels are not opaque.
            pixel[3] >= 64 && pixel[0] > 28 && pixel[0] > pixel[1] && pixel[1] > pixel[2]
        })
        .count()
}

fn dominant_red_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[3] >= 240 && pixel[0] > 72 && pixel[0] > pixel[1] + 32)
        .count()
}

fn dominant_green_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[3] >= 240 && pixel[1] > 72 && pixel[1] > pixel[0] + 32)
        .count()
}

fn dominant_blue_pixels(rgba: &[u8]) -> usize {
    rgba.chunks_exact(4)
        .filter(|pixel| pixel[3] >= 240 && pixel[2] > 72 && pixel[2] > pixel[0] + 32)
        .count()
}

fn average_channel(rgba: &[u8], channel: usize) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }
    let total = rgba
        .chunks_exact(4)
        .map(|pixel| pixel[channel] as f32)
        .sum::<f32>();
    total / (rgba.len() as f32 / 4.0)
}

fn average_channel_in_region(
    frame: &crate::core::framework::render::CapturedFrame,
    origin: UVec2,
    size: UVec2,
    channel: usize,
) -> f32 {
    let x_end = origin.x.saturating_add(size.x).min(frame.width) as usize;
    let y_end = origin.y.saturating_add(size.y).min(frame.height) as usize;
    let width = frame.width as usize;
    let mut total = 0.0;
    let mut count = 0.0;
    for y in origin.y as usize..y_end {
        for x in origin.x as usize..x_end {
            let index = (y * width + x) * 4 + channel;
            total += frame.rgba[index] as f32;
            count += 1.0;
        }
    }
    if count <= 0.0 {
        0.0
    } else {
        total / count
    }
}

fn write_flat_color_wgsl(path: PathBuf, color: [f32; 3]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        format!(
            "{GPU_SCENE_TEST_WGSL}\n{}",
            format!(
                r#"
struct SceneUniform {{
    view_proj: mat4x4<f32>,
}};

struct MaterialPropertyUniform {{
    data0: vec4<f32>,
    data1: vec4<f32>,
    data2: vec4<f32>,
    data3: vec4<f32>,
    data4: vec4<f32>,
    data5: vec4<f32>,
    data6: vec4<f32>,
    data7: vec4<f32>,
}};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;
@group(2) @binding(10) var<uniform> material_properties: MaterialPropertyUniform;

struct VertexInput {{
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}};

struct VertexOutput {{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
}};

@vertex
fn vs_main(input: VertexInput, @builtin(instance_index) instance_index: u32) -> VertexOutput {{
    var output: VertexOutput;
    let world = zr_world_from_local(instance_index) * vec4<f32>(input.position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.uv = input.uv;
    output.tint = zr_gpu_scene_tint(instance_index);
    return output;
}}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {{
    let alpha = textureSample(albedo_tex, albedo_sampler, input.uv).a;
    return vec4<f32>({:.6}, {:.6}, {:.6}, alpha) * input.tint;
}}
"#,
                color[0], color[1], color[2]
            )
        ),
    )
    .unwrap();
}

fn sample_texture_shader(uri: AssetUri) -> ShaderAsset {
    ShaderAsset {
        uri,
        source_language: ShaderSourceLanguage::Wgsl,
        source: sample_texture_wgsl_source(),
        wgsl_source: String::new(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: Vec::new(),
        texture_slots: Vec::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn sample_texture_wgsl_source() -> String {
    format!(
        "{GPU_SCENE_TEST_WGSL}\n{}",
        r#"
struct SceneUniform {
    view_proj: mat4x4<f32>,
};

struct MaterialPropertyUniform {
    data0: vec4<f32>,
    data1: vec4<f32>,
    data2: vec4<f32>,
    data3: vec4<f32>,
    data4: vec4<f32>,
    data5: vec4<f32>,
    data6: vec4<f32>,
    data7: vec4<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(2) @binding(0) var albedo_tex: texture_2d<f32>;
@group(2) @binding(1) var albedo_sampler: sampler;
@group(2) @binding(10) var<uniform> material_properties: MaterialPropertyUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput, @builtin(instance_index) instance_index: u32) -> VertexOutput {
    var output: VertexOutput;
    let world = zr_world_from_local(instance_index) * vec4<f32>(input.position, 1.0);
    output.clip_position = scene.view_proj * world;
    output.uv = input.uv;
    output.tint = zr_gpu_scene_tint(instance_index);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(albedo_tex, albedo_sampler, input.uv) * input.tint;
}
"#
    )
}

fn render_target_texture_descriptor() -> TextureAssetDescriptor {
    TextureAssetDescriptor {
        format: RGBA8_UNORM_FORMAT.to_string(),
        color_space: RenderImageColorSpace::Linear,
        sampler: RenderSamplerDescriptor::default(),
        usage: vec![
            RenderImageUsage::RenderTarget,
            RenderImageUsage::Sampled,
            RenderImageUsage::CopySrc,
        ],
        fallback: RenderImageFallbackKind::MissingImage,
        ..TextureAssetDescriptor::default()
    }
}

fn srgb_render_target_texture_descriptor() -> TextureAssetDescriptor {
    TextureAssetDescriptor {
        format: RGBA8_UNORM_SRGB_FORMAT.to_string(),
        color_space: RenderImageColorSpace::Srgb,
        ..render_target_texture_descriptor()
    }
}

trait CameraDescriptorTestExt {
    fn with_stack(self, stack: impl IntoIterator<Item = u64>) -> Self;
}

impl CameraDescriptorTestExt for CameraRenderDescriptor {
    fn with_stack(mut self, stack: impl IntoIterator<Item = u64>) -> Self {
        self.stack = stack.into_iter().collect();
        self
    }
}

fn write_solid_png(path: PathBuf, rgba: [u8; 4]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |_x, _y| Rgba(rgba))
        .save_with_format(path, ImageFormat::Png)
        .unwrap();
}

fn write_quad_obj(path: PathBuf) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(
        path,
        "\
v -1.0 -1.0 0.0
v 1.0 -1.0 0.0
v 1.0 1.0 0.0
v -1.0 1.0 0.0
vt 0.0 1.0
vt 1.0 1.0
vt 1.0 0.0
vt 0.0 0.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
f 1/1/1 3/3/1 4/4/1
",
    )
    .unwrap();
}

fn write_material_with_base_color_and_texture(
    path: PathBuf,
    shader_uri: &str,
    base_color: [f32; 4],
    base_color_texture: &str,
    alpha_mode: AlphaMode,
) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let material = MaterialAsset {
        name: Some("FlatColor".to_string()),
        shader: asset_reference(shader_uri),
        base_color,
        base_color_texture: Some(asset_reference(base_color_texture)),
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode,
        double_sided: false,
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    fs::write(path, material.to_toml_string().unwrap()).unwrap();
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn resource_handle<T>(asset_manager: &ProjectAssetManager, uri: &str) -> ResourceHandle<T> {
    ResourceHandle::new(
        asset_manager
            .resolve_asset_id(&AssetUri::parse(uri).unwrap())
            .unwrap_or_else(|| panic!("missing resource id for {uri}")),
    )
}
